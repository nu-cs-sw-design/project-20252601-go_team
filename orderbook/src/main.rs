fn main() {

    let number_slices: &[i32] = &[1,2,3,4,5];
    println!("Number slices: {:?}", number_slices);

    let animals: &[&String] = &[&"dog".to_string(), &"cat".to_string(), &"mouse".to_string()];
    println!("Animals: {:?}", animals);

    let mut stone_cold: String = String::from("Hell ");
    stone_cold.push_str("Yeah!");
    println!("Stone Cold says: {}", stone_cold);

    let string: String = String::from("Hello, World!");
    let string_slice: &str = &string[0..5];
    println!("slice value is {}", string_slice);

    let camel_case: i32 = 123;
    println!("{}", camel_case);

    human_id("Joel", 32, 180.6);

    let x = {
        let price = 5;
        let qty = 10;
        price * qty
    };

    println!("result is {}", add(x, x));

    println!("bmi is {:.2}", calculate_bmi(74.23, 1.7673));
}

fn human_id(name: &str, age: u32, height: f32) {
    println!("My name is {}, I am {} years old, and my height is {}", name, age, height);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn calculate_bmi(weight_kg: f64, height_m: f64) -> f64 {
    weight_kg / (height_m * height_m)
}