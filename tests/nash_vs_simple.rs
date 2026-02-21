use smbc_incomplete::*;

use rand::Rng;

/// Synchronous SimpleAi move selection (no printing).
fn simple_move(table: &Table, obj: Objective, pos: Pos) -> Pos {
    *moves(pos)
        .iter()
        .max_by_key(|&m| pref(obj, cannon_lookup(table, *m)))
        .unwrap()
}

/// Synchronous NashAi move selection (no printing).
fn nash_move(strategy: &StrategyMap, type_id: TypeId, pos: Pos, rng: &mut impl Rng) -> Pos {
    use smbc_incomplete::Square::*;

    let sigma = strategy
        .get(&pos)
        .map(|s| s[type_id])
        .unwrap_or_else(|| {
            let n = pos.board.iter().filter(|&&s| s == Open).count() as f64;
            let mut s = [0.0; 9];
            for (i, &sq) in pos.board.iter().enumerate() {
                if sq == Open {
                    s[i] = 1.0 / n;
                }
            }
            s
        });

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

    let mut board2 = pos.board;
    board2[chosen_cell] = Taken { by: pos.turn };
    Pos {
        turn: other(pos.turn),
        board: board2,
    }
}

/// Play one game, Nash vs Nash. Returns the outcome.
fn play_nash_vs_nash(
    strategy: &StrategyMap,
    x_type: TypeId,
    o_type: TypeId,
    rng: &mut impl Rng,
) -> Outcome {
    let mut pos = START;
    loop {
        let type_id = if pos.turn == Player::X { x_type } else { o_type };
        let new_pos = nash_move(strategy, type_id, pos, rng);
        if let Some(outcome) = eval_pos(new_pos) {
            return outcome;
        }
        pos = new_pos;
    }
}

/// Play one game, Nash vs Simple. Returns the outcome.
fn play_game(
    table: &Table,
    strategy: &StrategyMap,
    nash_type: TypeId,
    simple_obj: Objective,
    nash_is_x: bool,
    rng: &mut impl Rng,
) -> Outcome {
    let mut pos = START;
    loop {
        let is_nash_turn = (pos.turn == Player::X) == nash_is_x;
        let new_pos = if is_nash_turn {
            nash_move(strategy, nash_type, pos, rng)
        } else {
            simple_move(table, simple_obj, pos)
        };

        if let Some(outcome) = eval_pos(new_pos) {
            return outcome;
        }
        pos = new_pos;
    }
}

#[test]
fn nash_beats_simple_decisively() {
    // Setup: generate eval table and solve CFR
    let table = gen_table();
    let dag = build_dag();
    let prior = [1.0 / NUM_TYPES as f64; NUM_TYPES];
    let strategy = solve_cfr(&dag, &prior, 5_000);

    let num_games = 1_000_000;
    let mut rng = rand::thread_rng();

    let mut nash_total_utility = 0.0;
    let mut simple_total_utility = 0.0;
    let mut nash_wins = 0u64;
    let mut simple_wins = 0u64;
    let mut draws = 0u64;

    for _ in 0..num_games {
        // Random type assignments
        let nash_type: TypeId = rng.gen_range(0..NUM_TYPES);
        let simple_type: TypeId = rng.gen_range(0..NUM_TYPES);
        let simple_obj = ALL_OBJECTIVES[simple_type];

        // Alternate who plays X
        let nash_is_x: bool = rng.gen();

        let outcome = play_game(&table, &strategy, nash_type, simple_obj, nash_is_x, &mut rng);

        let nash_obj = &ALL_OBJECTIVES[nash_type];
        let nash_u = outcome_utility(nash_obj, outcome);
        let simple_u = outcome_utility(&simple_obj, outcome);

        nash_total_utility += nash_u;
        simple_total_utility += simple_u;

        // Track who "won" (got their most-preferred outcome)
        match outcome {
            Outcome::Draw => draws += 1,
            Outcome::Win { with } => {
                let nash_player = if nash_is_x { Player::X } else { Player::O };
                if with == nash_player {
                    nash_wins += 1;
                } else {
                    simple_wins += 1;
                }
            }
        }
    }

    let nash_avg = nash_total_utility / num_games as f64;
    let simple_avg = simple_total_utility / num_games as f64;

    eprintln!("=== Nash vs Simple: {} games ===", num_games);
    eprintln!("Nash avg utility:   {:.4}", nash_avg);
    eprintln!("Simple avg utility: {:.4}", simple_avg);
    eprintln!(
        "Nash wins: {} ({:.1}%)",
        nash_wins,
        nash_wins as f64 / num_games as f64 * 100.0
    );
    eprintln!(
        "Simple wins: {} ({:.1}%)",
        simple_wins,
        simple_wins as f64 / num_games as f64 * 100.0
    );
    eprintln!(
        "Draws: {} ({:.1}%)",
        draws,
        draws as f64 / num_games as f64 * 100.0
    );
    eprintln!("Utility advantage: {:.4}", nash_avg - simple_avg);

    // Nash equilibrium strategy should outperform the simple greedy strategy
    assert!(
        nash_avg > simple_avg,
        "Nash strategy should have higher average utility than simple strategy. \
         Nash: {:.4}, Simple: {:.4}",
        nash_avg,
        simple_avg
    );

    // The margin should be meaningful, not just noise
    let margin = nash_avg - simple_avg;
    assert!(
        margin > 0.01,
        "Nash advantage should be decisive (>1% utility). Got: {:.4}",
        margin
    );
}

#[test]
fn empirical_payoffs_match_theoretical_matrix() {
    // Setup
    let dag = build_dag();
    let prior = [1.0 / NUM_TYPES as f64; NUM_TYPES];
    let strategy = solve_cfr(&dag, &prior, 5_000);

    // Theoretical payoff matrix from the CFR solver
    let theoretical = expected_payoffs(&strategy, &dag);

    let games_per_pair = 100_000;
    let mut rng = rand::thread_rng();

    // Accumulate empirical utilities per pair
    let mut empirical_x = [[0.0f64; NUM_TYPES]; NUM_TYPES];
    let mut empirical_o = [[0.0f64; NUM_TYPES]; NUM_TYPES];

    for tx in 0..NUM_TYPES {
        for to in 0..NUM_TYPES {
            for _ in 0..games_per_pair {
                let outcome = play_nash_vs_nash(&strategy, tx, to, &mut rng);
                empirical_x[tx][to] += outcome_utility(&ALL_OBJECTIVES[tx], outcome);
                empirical_o[tx][to] += outcome_utility(&ALL_OBJECTIVES[to], outcome);
            }
            empirical_x[tx][to] /= games_per_pair as f64;
            empirical_o[tx][to] /= games_per_pair as f64;
        }
    }

    // Print comparison table
    eprintln!("\n=== Payoff Matrix: Theoretical vs Empirical ({} games/pair) ===", games_per_pair);
    eprintln!("{:<12} {:>22}  {:>22}  {:>8}", "Pair", "Theoretical (X, O)", "Empirical (X, O)", "Max Err");
    eprintln!("{}", "-".repeat(70));

    let mut max_error = 0.0f64;
    let mut all_errors = Vec::new();

    for tx in 0..NUM_TYPES {
        for to in 0..NUM_TYPES {
            let pair_idx = tx * NUM_TYPES + to;
            let [theo_x, theo_o] = theoretical[pair_idx];
            let emp_x = empirical_x[tx][to];
            let emp_o = empirical_o[tx][to];
            let err_x = (theo_x - emp_x).abs();
            let err_o = (theo_o - emp_o).abs();
            let err = err_x.max(err_o);
            max_error = max_error.max(err);
            all_errors.push(err);

            eprintln!(
                "({}, {})      ({:.4}, {:.4})      ({:.4}, {:.4})      {:.4}",
                tx, to, theo_x, theo_o, emp_x, emp_o, err
            );
        }
    }

    let mean_error: f64 = all_errors.iter().sum::<f64>() / all_errors.len() as f64;
    eprintln!("{}", "-".repeat(70));
    eprintln!("Max error:  {:.4}", max_error);
    eprintln!("Mean error: {:.4}", mean_error);

    // With 100k games per pair, expected std dev is ~sqrt(var/n).
    // Utility is in [0, 1], so variance ≤ 0.25, std dev of mean ≤ 0.5/sqrt(100k) ≈ 0.0016.
    // Allow ~3 std devs (~0.005) plus some CFR convergence error.
    let tolerance = 0.02;
    for tx in 0..NUM_TYPES {
        for to in 0..NUM_TYPES {
            let pair_idx = tx * NUM_TYPES + to;
            let [theo_x, theo_o] = theoretical[pair_idx];
            let emp_x = empirical_x[tx][to];
            let emp_o = empirical_o[tx][to];

            assert!(
                (theo_x - emp_x).abs() < tolerance,
                "X utility mismatch for pair ({}, {}): theoretical {:.4}, empirical {:.4}",
                tx, to, theo_x, emp_x
            );
            assert!(
                (theo_o - emp_o).abs() < tolerance,
                "O utility mismatch for pair ({}, {}): theoretical {:.4}, empirical {:.4}",
                tx, to, theo_o, emp_o
            );
        }
    }
}
