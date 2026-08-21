#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Grab the username from the caller's environment, or die warning them.
    let Ok(username) = dotenv::var("AASMS_USER") else {
        println!("Specify AASMS_USER!!");
        return Ok(());
    };

    // Grab the password from the caller's environment, or die warning them.
    let Ok(password) = dotenv::var("AASMS_PASS") else {
        println!("Specify AASMS_PASS!!");
        return Ok(());
    };

    // Grab the destination from the caller's environment, or die warning them.
    let Ok(destination) = dotenv::var("AASMS_DEST") else {
        println!("Specify AASMS_DEST!!");
        return Ok(());
    };

    // We're hardcoding a message for this demo. Since the API accepts UTF-8 strings, we can send
    // some emojis too!
    let message = String::from("⌨ This is a test! 📱");

    // Build a client to post to the A&A SMS API with.
    let client = aa_sms::Client::builder()
        .username(username)
        .password(password)
        .build();

    // Build our message.
    let sms = aa_sms::Message::builder()
        .destination(destination)
        .message(message)
        .build();

    // Post our message to the A&A SMS API.
    client.send(sms).await?;

    Ok(())
}
