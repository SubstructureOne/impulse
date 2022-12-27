use self::models::Report;
use impulse::establish_connection;
use diesel::prelude::*;

use impulse::*;

fn main() {
    // use self::schema::reports::dsl::*;
    let conn = &mut establish_connection();
    let results = schema::reports::table
        .limit(5)
        .load::<Report>(conn)
        .expect("Error loading reports");
    println!("Displaying {} reports", results.len());
    for report in results {
        println!("{:?}", report);
    }
}
