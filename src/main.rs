use smbc_incomplete::Outcome::*;
use smbc_incomplete::Player::*;
use smbc_incomplete::*;

use tokio::runtime::Runtime;

fn main() {
    let table = gen_table();
    let runtime = Runtime::new().expect("hurr I can fail?");
    let _rt_ent = runtime.enter();
    let mut _cnsl = Cnsl(&table);
    let mut ai = SimpleAi
        {table:&table
        ,obj:[Win { with: O }, Win { with: X }, Draw]
        };
    runtime.block_on(async{
        let mut weeb = mkWebPlayer().await;
        host_game(&mut ai,&mut weeb,START).await;
    });
}

