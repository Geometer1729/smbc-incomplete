use smbc_incomplete::bayesian;
use smbc_incomplete::*;

use rand::Rng;
use std::sync::Arc;

fn format_objective(obj: &Objective) -> String {
    let names: Vec<&str> = obj
        .iter()
        .map(|o| match o {
            Outcome::Win { with: Player::X } => "X wins",
            Outcome::Draw => "Draw",
            Outcome::Win { with: Player::O } => "O wins",
        })
        .collect();
    format!("{} > {} > {}", names[0], names[1], names[2])
}

fn format_outcome(o: Outcome) -> &'static str {
    match o {
        Outcome::Win { with: Player::X } => "X wins",
        Outcome::Draw => "Draw",
        Outcome::Win { with: Player::O } => "O wins",
    }
}

fn score_label(obj: &Objective, outcome: Outcome) -> &'static str {
    if outcome == obj[0] {
        "Best"
    } else if outcome == obj[1] {
        "Middle"
    } else {
        "Worst"
    }
}

#[tokio::main]
async fn main() {
    // Suppress "connection closed" panic messages from WebSocket disconnects
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let is_disconnect = info
            .payload()
            .downcast_ref::<&str>()
            .is_some_and(|&msg| msg == "connection closed");
        if !is_disconnect {
            default_hook(info);
        }
    }));

    let table = gen_table();

    // Build DAG and solve Nash equilibrium via CFR
    let prior = [1.0 / 6.0; bayesian::NUM_TYPES];
    let iterations = 5000;

    let strategy = if let Some(cached) = bayesian::load_cached_strategy(&prior, iterations) {
        println!("Loaded cached strategy from disk.");
        Arc::new(cached)
    } else {
        println!("Building game DAG...");
        let dag = bayesian::build_dag();
        println!("DAG built: {} reachable positions", dag.num_positions());

        println!("Running CFR for {} iterations...", iterations);
        let start = std::time::Instant::now();
        let strategy = bayesian::solve_cfr(&dag, &prior, iterations);
        println!("CFR complete in {:.2}s.", start.elapsed().as_secs_f64());

        bayesian::save_cached_strategy(&strategy, &prior, iterations);
        println!("Strategy cached to disk.");
        Arc::new(strategy)
    };

    println!("Building game DAG for payoff analysis...");
    let dag = bayesian::build_dag();

    let payoffs = bayesian::expected_payoffs(&strategy, &dag);
    bayesian::print_payoff_matrix(&payoffs);

    let addr = "127.0.0.1:8080";
    let listener = bind_listener(addr).await;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, peer) = listener.accept().await.expect("Failed to accept");
        let strategy = strategy.clone();
        let table = table.clone();
        tokio::spawn(async move {
            let mut weeb = match handle_connection(stream).await {
                Some(wp) => wp,
                None => return,
            };
            println!("[{}] Connected", peer);

            'outer: loop {
                let (ai_type, human_type) = {
                    let mut rng = rand::thread_rng();
                    loop {
                        let ai = rng.gen_range(0..bayesian::NUM_TYPES);
                        let human = rng.gen_range(0..bayesian::NUM_TYPES);
                        if bayesian::ALL_OBJECTIVES[ai][0] != bayesian::ALL_OBJECTIVES[human][0] {
                            break (ai, human);
                        }
                    }
                };
                let human_obj = bayesian::ALL_OBJECTIVES[human_type];
                let ai_obj = bayesian::ALL_OBJECTIVES[ai_type];

                println!(
                    "[{}] New game: Human ({}) vs AI ({})",
                    peer,
                    format_objective(&human_obj),
                    format_objective(&ai_obj)
                );

                let mut nash_ai = bayesian::NashAi::new(strategy.clone(), ai_type);

                weeb.cheat = Some(CheatState {
                    table: table.clone(),
                    strategy: strategy.clone(),
                    human_type,
                    human_obj,
                });

                weeb.send_info(format!("OBJ:{}", format_objective(&human_obj)))
                    .await;

                let outcome =
                    std::panic::AssertUnwindSafe(host_game(&mut weeb, &mut nash_ai, START));
                let outcome = match futures_util::FutureExt::catch_unwind(outcome).await {
                    Ok(o) => o,
                    Err(_) => {
                        println!("[{}] Disconnected", peer);
                        break 'outer;
                    }
                };

                weeb.send_info(format!(
                    "END:{}|{}|{}|{}|{}",
                    format_outcome(outcome),
                    format_objective(&human_obj),
                    score_label(&human_obj, outcome),
                    format_objective(&ai_obj),
                    score_label(&ai_obj, outcome),
                ))
                .await;

                println!(
                    "[{}] Result: {} — Human: {}, AI: {}",
                    peer,
                    format_outcome(outcome),
                    score_label(&human_obj, outcome),
                    score_label(&ai_obj, outcome)
                );

                weeb.wait_for_reset().await;
                println!("[{}] Restarting...", peer);
            }
        });
    }
}
