use macros::execute;


#[derive(PartialEq, Debug)]
struct User;

fn main() {
    let x = "ads".to_string();
        let result = execute!(
        User,
        "This is a test message",
        (1, 2, "ASD"),
    );

    assert_eq!(result, (User, "This is a test message", (1, 2, "ASD")));
}
