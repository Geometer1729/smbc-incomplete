use crate::types::*;
use crate::api::API;

pub struct WebPlayer<'a>
    {pub table:&'a Table
    }

impl API for WebPlayer<'_> {
    fn rend(&self,_p:Pos){
        todo!("implement rend")
    }

    fn ask(&self,_p:Pos) -> Pos {
        todo!("implement ask")
    }
}
