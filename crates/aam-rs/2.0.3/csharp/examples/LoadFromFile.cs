using AamCsharp;

Console.WriteLine("=== C# AAM Configuration Example ===\n");

try
{
    // Load configuration from file
    Console.WriteLine("Loading configuration from config.aam...");
    using var config = AamDocument.Load("config.aam");

    Console.WriteLine("\n--- Application Info ---");
    Console.WriteLine($"Application: {config.Get("app_name")} v{config.Get("app_version")}");
    Console.WriteLine($"Environment: {config.Get("environment")}");

    Console.WriteLine("\n--- Server Configuration ---");
    Console.WriteLine($"Host: {config.Get("server_host")}");
    Console.WriteLine($"Port: {config.Get("server_port")}");
    Console.WriteLine($"Timeout: {config.Get("server_timeout")}ms");

    Console.WriteLine("\n--- Database Configuration ---");
    Console.WriteLine($"Type: {config.Get("db_type")}");
    Console.WriteLine($"Host: {config.Get("db_host")}:{config.Get("db_port")}");
    Console.WriteLine($"Database: {config.Get("db_name")}");
    Console.WriteLine($"Max Connections: {config.Get("db_max_connections")}");

    Console.WriteLine("\n--- Logging Settings ---");
    Console.WriteLine($"Level: {config.Get("log_level")}");
    Console.WriteLine($"Format: {config.Get("log_format")}");
    Console.WriteLine($"Output: {config.Get("log_output")}");

    Console.WriteLine("\n--- Feature Flags ---");
    Console.WriteLine($"Analytics: {config.Get("feature_analytics")}");
    Console.WriteLine($"Caching: {config.Get("feature_caching")}");
    Console.WriteLine($"Debug Mode: {config.Get("feature_debug_mode")}");

    Console.WriteLine("\n--- Query by value with Find(...) ---");
    var envMatches = config.Find("production");
    foreach (var entry in envMatches)
    {
        Console.WriteLine($"{entry.Key}: {entry.Value}");
    }
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Error: Native library not found - {ex.Message}");
    Console.WriteLine("Please ensure the aam_rs native library is available in your PATH.");
}
catch (InvalidOperationException ex)
{
    Console.WriteLine($"Error parsing configuration: {ex.Message}");
}
catch (FileNotFoundException ex)
{
    Console.WriteLine($"Error: Configuration file not found - {ex.Message}");
}

Console.WriteLine("\nExample completed!");

