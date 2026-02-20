use crate::api::API;
use crate::eval::eval_pos;
use crate::moves::{other, START};
use crate::types::Outcome::*;
use crate::types::Player::*;
use crate::types::Square::*;
use crate::types::*;

use std::collections::{HashMap, VecDeque};

use rand::Rng;

// ---------------------------------------------------------------------------
// Foundation types
// ---------------------------------------------------------------------------

pub const NUM_TYPES: usize = 6;
pub type TypeId = usize; // 0..6

/// All 6 preference orderings over {X-wins, Draw, O-wins}.
pub const ALL_OBJECTIVES: [Objective; NUM_TYPES] = [
    [Win { with: X }, Draw, Win { with: O }],
    [Win { with: X }, Win { with: O }, Draw],
    [Draw, Win { with: X }, Win { with: O }],
    [Draw, Win { with: O }, Win { with: X }],
    [Win { with: O }, Win { with: X }, Draw],
    [Win { with: O }, Draw, Win { with: X }],
];

/// Map Outcome to index: X-wins=0, Draw=1, O-wins=2
pub fn outcome_to_index(o: Outcome) -> usize {
    match o {
        Win { with: X } => 0,
        Draw => 1,
        Win { with: O } => 2,
    }
}

/// Numeric utility for a type: rank 0 (most preferred) → 1.0, rank 1 → 0.5, rank 2 → 0.0
pub fn utility_values(obj: &Objective) -> [f64; 3] {
    let mut vals = [0.0; 3];
    for (rank, &outcome) in obj.iter().enumerate() {
        let idx = outcome_to_index(outcome);
        vals[idx] = match rank {
            0 => 1.0,
            1 => 0.5,
            _ => 0.0,
        };
    }
    vals
}

/// Return the utility for a specific outcome given an objective.
fn outcome_utility(obj: &Objective, outcome: Outcome) -> f64 {
    let vals = utility_values(obj);
    vals[outcome_to_index(outcome)]
}

// ---------------------------------------------------------------------------
// Pre-computed terminal utilities for all 36 type pairs × 3 outcomes
// ---------------------------------------------------------------------------

/// TERMINAL_UTILS[outcome_idx][tx * 6 + to] = [util_x, util_o]
fn precompute_terminal_utils() -> [[[f64; 2]; 36]; 3] {
    let mut table = [[[0.0; 2]; 36]; 3];
    let outcomes = [Win { with: X }, Draw, Win { with: O }];
    for (oi, &outcome) in outcomes.iter().enumerate() {
        for tx in 0..NUM_TYPES {
            for to in 0..NUM_TYPES {
                table[oi][tx * NUM_TYPES + to] = [
                    outcome_utility(&ALL_OBJECTIVES[tx], outcome),
                    outcome_utility(&ALL_OBJECTIVES[to], outcome),
                ];
            }
        }
    }
    table
}

// ---------------------------------------------------------------------------
// Game DAG (index-based for performance)
// ---------------------------------------------------------------------------

/// Index of a position in the flat array.
type PosIdx = usize;

/// Pre-computed info about a position.
struct PosInfo {
    pos: Pos,
    /// None = non-terminal, Some(outcome_index) = terminal
    terminal: Option<usize>,
    /// true if mover is X
    mover_is_x: bool,
    /// Children as (cell_index, child_pos_idx)
    children: Vec<(usize, PosIdx)>,
}

pub struct GameDag {
    nodes: Vec<PosInfo>,
    /// Indices into `nodes` grouped by depth.
    by_depth: Vec<Vec<PosIdx>>,
}

impl GameDag {
    pub fn num_positions(&self) -> usize {
        self.nodes.len()
    }
}

/// Count filled squares on a board.
fn depth_of(pos: &Pos) -> usize {
    pos.board.iter().filter(|&&s| s != Open).count()
}

/// Build the DAG of all reachable positions from START.
pub fn build_dag() -> GameDag {
    let mut visited: HashMap<Pos, PosIdx> = HashMap::new();
    let mut queue: VecDeque<Pos> = VecDeque::new();
    let mut all_positions: Vec<Pos> = Vec::new();

    let start_idx = 0;
    visited.insert(START, start_idx);
    all_positions.push(START);
    queue.push_back(START);

    // BFS to discover all positions and assign indices
    while let Some(pos) = queue.pop_front() {
        if eval_pos(pos).is_some() {
            continue;
        }
        for i in 0..9 {
            if pos.board[i] == Open {
                let mut board2 = pos.board;
                board2[i] = Taken { by: pos.turn };
                let child = Pos {
                    turn: other(pos.turn),
                    board: board2,
                };
                if let std::collections::hash_map::Entry::Vacant(e) = visited.entry(child) {
                    let idx = all_positions.len();
                    e.insert(idx);
                    all_positions.push(child);
                    queue.push_back(child);
                }
            }
        }
    }

    let n = all_positions.len();

    // Build PosInfo for each position
    let outcomes = [Win { with: X }, Draw, Win { with: O }];
    let mut nodes: Vec<PosInfo> = Vec::with_capacity(n);
    for &pos in &all_positions {
        let terminal = eval_pos(pos).map(|o| outcomes.iter().position(|&x| x == o).unwrap());
        let mover_is_x = pos.turn == X;
        let mut children = Vec::new();
        if terminal.is_none() {
            for i in 0..9 {
                if pos.board[i] == Open {
                    let mut board2 = pos.board;
                    board2[i] = Taken { by: pos.turn };
                    let child = Pos {
                        turn: other(pos.turn),
                        board: board2,
                    };
                    children.push((i, visited[&child]));
                }
            }
        }
        nodes.push(PosInfo {
            pos,
            terminal,
            mover_is_x,
            children,
        });
    }

    // Group by depth
    let mut by_depth = vec![Vec::new(); 10];
    for (idx, pos) in all_positions.iter().enumerate() {
        by_depth[depth_of(pos)].push(idx);
    }

    GameDag { nodes, by_depth }
}

// ---------------------------------------------------------------------------
// CFR Solver (index-based, no HashMap in hot loop)
// ---------------------------------------------------------------------------

type ActionProbs = [f64; 9];

/// Strategy profile indexed by position.
pub type StrategyMap = HashMap<Pos, [ActionProbs; NUM_TYPES]>;

/// Convert cumulative regrets → strategy via regret matching.
fn regret_match(regrets: &ActionProbs, children: &[(usize, PosIdx)]) -> ActionProbs {
    let mut strategy = [0.0; 9];
    let mut pos_sum = 0.0;

    for &(cell, _) in children {
        let r = regrets[cell].max(0.0);
        strategy[cell] = r;
        pos_sum += r;
    }

    if pos_sum > 0.0 {
        for &(cell, _) in children {
            strategy[cell] /= pos_sum;
        }
    } else {
        let n = children.len() as f64;
        for &(cell, _) in children {
            strategy[cell] = 1.0 / n;
        }
    }

    strategy
}

/// Solve the game using Counterfactual Regret Minimization.
pub fn solve_cfr(dag: &GameDag, prior: &[f64; NUM_TYPES], iterations: usize) -> StrategyMap {
    let n = dag.nodes.len();
    let max_depth = dag.by_depth.len() - 1;
    let terminal_utils = precompute_terminal_utils();

    // Flat arrays indexed by pos_idx
    // For non-terminal positions: [NUM_TYPES][9]
    let mut cum_regret: Vec<[[f64; 9]; NUM_TYPES]> = vec![[[0.0; 9]; NUM_TYPES]; n];
    let mut cum_strategy: Vec<[[f64; 9]; NUM_TYPES]> = vec![[[0.0; 9]; NUM_TYPES]; n];
    let mut current_strategy: Vec<[[f64; 9]; NUM_TYPES]> = vec![[[0.0; 9]; NUM_TYPES]; n];

    // Values and reach per iteration (reused)
    let mut values: Vec<[[f64; 2]; 36]> = vec![[[0.0; 2]; 36]; n];
    let mut cf_reach: Vec<[[f64; 2]; 36]> = vec![[[0.0; 2]; 36]; n];

    for _iter in 0..iterations {
        // 1. Regret-match → current strategy (only non-terminal)
        for (idx, node) in dag.nodes.iter().enumerate() {
            if node.terminal.is_some() {
                continue;
            }
            for t in 0..NUM_TYPES {
                current_strategy[idx][t] = regret_match(&cum_regret[idx][t], &node.children);
            }
        }

        // 2. Value pass (bottom-up)
        for d in (0..=max_depth).rev() {
            for &idx in &dag.by_depth[d] {
                let node = &dag.nodes[idx];
                if let Some(oi) = node.terminal {
                    values[idx] = terminal_utils[oi];
                } else {
                    let strats = &current_strategy[idx];
                    let mover_is_x = node.mover_is_x;
                    let mut v = [[0.0; 2]; 36];

                    for tx in 0..NUM_TYPES {
                        for to in 0..NUM_TYPES {
                            let pair_idx = tx * NUM_TYPES + to;
                            let mover_type = if mover_is_x { tx } else { to };
                            let sigma = &strats[mover_type];

                            let mut vx = 0.0;
                            let mut vo = 0.0;
                            for &(cell, child_idx) in &node.children {
                                let prob = sigma[cell];
                                if prob > 0.0 {
                                    let cv = &values[child_idx][pair_idx];
                                    vx += prob * cv[0];
                                    vo += prob * cv[1];
                                }
                            }
                            v[pair_idx] = [vx, vo];
                        }
                    }
                    values[idx] = v;
                }
            }
        }

        // 3. Reach pass (top-down)
        // Reset all reach to zero
        for r in cf_reach.iter_mut() {
            *r = [[0.0; 2]; 36];
        }

        // Root reach
        for tx in 0..NUM_TYPES {
            for to in 0..NUM_TYPES {
                let pair_idx = tx * NUM_TYPES + to;
                cf_reach[0][pair_idx][0] = prior[to]; // X's cf_reach = opponent's prior
                cf_reach[0][pair_idx][1] = prior[tx]; // O's cf_reach = opponent's prior
            }
        }

        for d in 0..=max_depth {
            for &idx in &dag.by_depth[d] {
                let node = &dag.nodes[idx];
                if node.terminal.is_some() {
                    continue;
                }

                let strats = &current_strategy[idx];
                let mover_is_x = node.mover_is_x;
                let parent_reach = cf_reach[idx];

                for &(cell, child_idx) in &node.children {
                    for tx in 0..NUM_TYPES {
                        for to in 0..NUM_TYPES {
                            let pair_idx = tx * NUM_TYPES + to;
                            let mover_type = if mover_is_x { tx } else { to };
                            let sigma_a = strats[mover_type][cell];

                            if mover_is_x {
                                // mover=X: cf_reach_X += parent × σ, cf_reach_O += parent × 1
                                cf_reach[child_idx][pair_idx][0] +=
                                    parent_reach[pair_idx][0] * sigma_a;
                                cf_reach[child_idx][pair_idx][1] += parent_reach[pair_idx][1];
                            } else {
                                // mover=O: cf_reach_O += parent × σ, cf_reach_X += parent × 1
                                cf_reach[child_idx][pair_idx][1] +=
                                    parent_reach[pair_idx][1] * sigma_a;
                                cf_reach[child_idx][pair_idx][0] += parent_reach[pair_idx][0];
                            }
                        }
                    }
                }
            }
        }

        // 4. Regret update + strategy accumulation
        for d in 0..=max_depth {
            for &idx in &dag.by_depth[d] {
                let node = &dag.nodes[idx];
                if node.terminal.is_some() {
                    continue;
                }

                let strats = &current_strategy[idx];
                let mover_is_x = node.mover_is_x;
                let mover_idx: usize = if mover_is_x { 0 } else { 1 };
                let pos_reach = &cf_reach[idx];

                for t in 0..NUM_TYPES {
                    let sigma = &strats[t];

                    // Compute cf-weighted value for each action
                    let mut action_values = [0.0f64; 9];
                    for &(cell, child_idx) in &node.children {
                        let child_v = &values[child_idx];
                        let mut weighted_v = 0.0;
                        for t_opp in 0..NUM_TYPES {
                            let pair_idx = if mover_is_x {
                                t * NUM_TYPES + t_opp
                            } else {
                                t_opp * NUM_TYPES + t
                            };
                            weighted_v +=
                                pos_reach[pair_idx][mover_idx] * child_v[pair_idx][mover_idx];
                        }
                        action_values[cell] = weighted_v;
                    }

                    // Current strategy value
                    let mut strategy_value = 0.0;
                    for &(cell, _) in &node.children {
                        strategy_value += sigma[cell] * action_values[cell];
                    }

                    // Update cumulative regrets
                    for &(cell, _) in &node.children {
                        cum_regret[idx][t][cell] += action_values[cell] - strategy_value;
                    }

                    // Accumulate average strategy (uniform weighting per iteration)
                    for i in 0..9 {
                        cum_strategy[idx][t][i] += sigma[i];
                    }
                }
            }
        }
    }

    // Normalize cumulative strategy → average strategy, convert to HashMap for NashAi
    let mut avg_strategy: StrategyMap = HashMap::new();
    for (idx, node) in dag.nodes.iter().enumerate() {
        if node.terminal.is_some() {
            continue;
        }
        let mut strats = [[0.0; 9]; NUM_TYPES];
        for t in 0..NUM_TYPES {
            let sum: f64 = cum_strategy[idx][t].iter().sum();
            if sum > 0.0 {
                for i in 0..9 {
                    strats[t][i] = cum_strategy[idx][t][i] / sum;
                }
            } else {
                let n = node.children.len() as f64;
                for &(cell, _) in &node.children {
                    strats[t][cell] = 1.0 / n;
                }
            }
        }
        avg_strategy.insert(node.pos, strats);
    }

    avg_strategy
}

// ---------------------------------------------------------------------------
// Equilibrium Analysis
// ---------------------------------------------------------------------------

/// Compute expected payoffs for each (tx, to) type pair under the given strategy.
pub fn expected_payoffs(strategy: &StrategyMap, dag: &GameDag) -> [[f64; 2]; 36] {
    let max_depth = dag.by_depth.len() - 1;
    let terminal_utils = precompute_terminal_utils();

    let n = dag.nodes.len();
    let mut values: Vec<[[f64; 2]; 36]> = vec![[[0.0; 2]; 36]; n];

    for d in (0..=max_depth).rev() {
        for &idx in &dag.by_depth[d] {
            let node = &dag.nodes[idx];
            if let Some(oi) = node.terminal {
                values[idx] = terminal_utils[oi];
            } else {
                let strats = &strategy[&node.pos];
                let mover_is_x = node.mover_is_x;

                let mut v = [[0.0; 2]; 36];
                for tx in 0..NUM_TYPES {
                    for to in 0..NUM_TYPES {
                        let pair_idx = tx * NUM_TYPES + to;
                        let mover_type = if mover_is_x { tx } else { to };
                        let sigma = &strats[mover_type];

                        for &(cell, child_idx) in &node.children {
                            let prob = sigma[cell];
                            if prob > 0.0 {
                                let cv = &values[child_idx][pair_idx];
                                v[pair_idx][0] += prob * cv[0];
                                v[pair_idx][1] += prob * cv[1];
                            }
                        }
                    }
                }
                values[idx] = v;
            }
        }
    }

    values[0] // START is index 0
}

/// Print the 6×6 payoff matrix.
pub fn print_payoff_matrix(payoffs: &[[f64; 2]; 36]) {
    println!("\n=== Nash Equilibrium Payoff Matrix ===");
    println!("Rows = X's type, Columns = O's type");
    println!("Each cell: (X payoff, O payoff)\n");

    // Header
    print!("{:>30}", "");
    for (to, obj) in ALL_OBJECTIVES.iter().enumerate() {
        print!("  O-type {} {:>14}", to, format!("{:?}", obj));
    }
    println!();

    for (tx, obj_x) in ALL_OBJECTIVES.iter().enumerate() {
        print!("X-type {} {:>20}", tx, format!("{:?}", obj_x));
        for to in 0..NUM_TYPES {
            let pair_idx = tx * NUM_TYPES + to;
            let [vx, vo] = payoffs[pair_idx];
            print!("  ({:.3}, {:.3})      ", vx, vo);
        }
        println!();
    }
    println!();
}

/// Print positions where the mixed strategy has significant entropy.
pub fn print_interesting_positions(strategy: &StrategyMap, dag: &GameDag) {
    println!("=== Positions with Significant Mixing ===\n");

    let mut interesting: Vec<(Pos, TypeId, f64)> = Vec::new();

    for node in &dag.nodes {
        if let Some(strats) = strategy.get(&node.pos) {
            for (t, sigma) in strats.iter().enumerate() {
                let entropy: f64 = sigma
                    .iter()
                    .filter(|&&p| p > 0.0)
                    .map(|&p| -p * p.ln())
                    .sum();
                if entropy > 0.3 {
                    interesting.push((node.pos, t, entropy));
                }
            }
        }
    }

    interesting.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    for (pos, t, entropy) in interesting.iter().take(20) {
        let strats = &strategy[pos];
        let sigma = &strats[*t];
        let moves_str: Vec<String> = sigma
            .iter()
            .enumerate()
            .filter(|(_, &p)| p > 0.01)
            .map(|(i, &p)| format!("cell {}={:.1}%", i, p * 100.0))
            .collect();
        println!(
            "depth={} turn={:?} type={} entropy={:.3}: {}",
            depth_of(pos),
            pos.turn,
            t,
            entropy,
            moves_str.join(", ")
        );
    }
    println!();
}

// ---------------------------------------------------------------------------
// Nash AI Player
// ---------------------------------------------------------------------------

pub struct NashAi {
    strategy: StrategyMap,
    my_type: TypeId,
}

impl NashAi {
    pub fn new(strategy: StrategyMap, my_type: TypeId) -> Self {
        NashAi { strategy, my_type }
    }
}

impl API for NashAi {
    async fn ask(&mut self, pos: Pos) -> Pos {
        let sigma = match self.strategy.get(&pos) {
            Some(strats) => strats[self.my_type],
            None => {
                // Fallback: uniform over legal moves
                let n = pos.board.iter().filter(|&&s| s == Open).count() as f64;
                let mut s = [0.0; 9];
                for (i, &sq) in pos.board.iter().enumerate() {
                    if sq == Open {
                        s[i] = 1.0 / n;
                    }
                }
                s
            }
        };

        // Sample from the mixed strategy
        let mut rng = rand::thread_rng();
        let roll: f64 = rng.gen();
        let mut cumulative = 0.0;
        let mut chosen_cell = 0;

        for (i, &prob) in sigma.iter().enumerate() {
            cumulative += prob;
            if roll < cumulative {
                chosen_cell = i;
                break;
            }
        }

        // Build the resulting position
        let mut board2 = pos.board;
        board2[chosen_cell] = Taken { by: pos.turn };
        let new_pos = Pos {
            turn: other(pos.turn),
            board: board2,
        };

        println!(
            "NashAi (type {}) selected cell {} (prob {:.1}%)",
            self.my_type,
            chosen_cell,
            sigma[chosen_cell] * 100.0
        );

        new_pos
    }
}
