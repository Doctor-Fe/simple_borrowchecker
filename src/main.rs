use evaluater::Evaluater;

mod evaluater;
mod parser;

fn main() {
    let mut eval: Evaluater = Evaluater::new();
    loop {
        let mut s: String = String::new();
        _ = std::io::stdin().read_line(&mut s);
        eval.split_elements(s);
        let mut t = eval.pop_command().unwrap();
        let data = parser::ExprParser::parse(&mut eval, &mut t);
        println!("{:?}", data);
    }
}
