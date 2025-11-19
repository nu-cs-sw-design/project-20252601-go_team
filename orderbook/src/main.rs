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

    
}

fn print() {
    println!("hi");
}

fn human_id(name: &str, age: u32, height: f32) {
    println!("My name is {}, I am {} years old, and my height is {}", name, age, height);
}