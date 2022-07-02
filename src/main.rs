use smbc_incomplete::Outcome::*;
use smbc_incomplete::Player::*;
use smbc_incomplete::*;

fn main() {
    let table = gen_table();
    println!("{}",Pref::Winable > Pref::Drawable);
    println!("{}",Pref::Drawable > Pref::Lost);
    println!("{}",Pref::Winable > Pref::Lost);
    let x = Cnsl(&table);
    let o = SimpleAi
        {table:&table
        ,obj:[Win { with: O }, Win { with: X }, Draw]
        };
    println!("{}",host_game(x,o,START));
}

