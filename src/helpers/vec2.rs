use vecmath::Vector2;

pub fn vec2_floor(vector: Vector2<f64>) -> Vector2<i32> {
    let [x, y] = vector;
    
    [x.floor() as i32, y.floor() as i32]
}