using AamCsharp;

var scenario = args.Length > 0 ? args[0].ToLowerInvariant() : "basic";

try
{
    switch (scenario)
    {
        case "basic":
            RunBasic();
            break;
        case "load":
            RunLoad();
            break;
        default:
            Console.WriteLine("Unknown scenario. Use: basic or load");
            Environment.ExitCode = 1;
            break;
    }
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
    Console.WriteLine("Build Rust with --features ffi and ensure the runtime library is discoverable.");
    Environment.ExitCode = 2;
}
catch (AamException ex)
{
    Console.WriteLine($"AAM error: {ex.Message}");
    Environment.ExitCode = 3;
}

static void RunBasic()
{
    using var doc = AamDocument.Parse("host = localhost\nport = 8080");
    Console.WriteLine($"host={doc.Get("host")}");
    Console.WriteLine($"port={doc.Get("port")}");
}

static void RunLoad()
{
    using var doc = AamDocument.Load("config.aam");
    Console.WriteLine($"app_name={doc.Get("app_name")}");
    Console.WriteLine($"environment={doc.Get("environment")}");
}

