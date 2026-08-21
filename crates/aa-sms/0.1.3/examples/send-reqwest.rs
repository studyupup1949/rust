/// The bare minimum that A&A's SMS API requires to send a message.
#[derive(serde::Serialize)]
struct Sms {
    /// Internationally formatted phone number
    username: String,

    /// VoIP password for username
    password: String,

    /// TP-DA, otherwise known as Destination Address
    da: String,

    /// TP-UD, otherwise known as User Data or Message
    ud: String,
}

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

    // Build our SMS payload.
    let sms = Sms {
        username,
        password,
        da: destination,
        ud: message,
    };

    // Post our payload as a JSON body to the A&A SMS API.
    let client = reqwest::Client::new();
    let res = client
        .post("https://sms.aa.net.uk/sms.cgi")
        .json(&sms)
        .send()
        .await?;

    // Grab and report the response body status code and response body text.
    let body = res.text().await?;
    println!("Body: {}", body);

    Ok(())
}
