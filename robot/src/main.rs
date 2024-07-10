use ice_cream_shop::shop_values::N_ROBOTS;
use robot::robot::Robot;
use std::env;

#[actix_rt::main]
async fn main() -> Result<(), &'static str> {
    let robot_id: u16 = env::args()
        .nth(1)
        .ok_or("args: <id>")?
        .parse()
        .map_err(|_| "The provided id needs to be a numeric value")?;

    if N_ROBOTS <= robot_id {
        return Err("The provided id is out of range");
    }

    Robot::spawn(robot_id)
        .await?
        .await
        .map_err(|_| "New connections receiver failed")?;

    Ok(())
}
