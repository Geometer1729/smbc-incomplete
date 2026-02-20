use smbc_incomplete::bayesian;
use smbc_incomplete::Outcome::*;
use smbc_incomplete::Player::*;
use smbc_incomplete::*;

use tokio::runtime::Runtime;

fn main() {
    let table = gen_table();

    // Build DAG and solve Nash equilibrium via CFR
    println!("Building game DAG...");
    let dag = bayesian::build_dag();
    println!("DAG built: {} reachable positions", dag.num_positions());

    let prior = [1.0 / 6.0; bayesian::NUM_TYPES];
    let iterations = 5000;
    println!("Running CFR for {} iterations...", iterations);
    let start = std::time::Instant::now();
    let strategy = bayesian::solve_cfr(&dag, &prior, iterations);
    println!("CFR complete in {:.2}s.", start.elapsed().as_secs_f64());

    // Print analysis
    let payoffs = bayesian::expected_payoffs(&strategy, &dag);
    bayesian::print_payoff_matrix(&payoffs);
    bayesian::print_interesting_positions(&strategy, &dag);

    let runtime = Runtime::new().expect("hurr I can fail?");
    let _rt_ent = runtime.enter();
    let mut cnsl = Cnsl(&table);
    let mut _ai = SimpleAi {
        table: &table,
        obj: [Win { with: O }, Win { with: X }, Draw],
    };
    let mut _nash_ai = bayesian::NashAi::new(strategy, 0);
    runtime.block_on(async {
        let mut weeb = mk_web_player().await;
        host_game(&mut cnsl, &mut weeb, START).await;
    });
}
