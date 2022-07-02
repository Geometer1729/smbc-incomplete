use smbc_incomplete::Outcome::*;
use smbc_incomplete::Player::*;
use smbc_incomplete::*;

use tokio::runtime::Runtime;

fn main() {
    let table = gen_table();
    //let mut x = Cnsl(&table);
    let mut x = Runtime::new().expect("hurr I can fail?").block_on(mkWebPlayer());
    let mut o = SimpleAi
        {table:&table
        ,obj:[Win { with: O }, Win { with: X }, Draw]
        };
    println!("{}",host_game(&mut x,&mut o,START));
}

