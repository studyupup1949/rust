using AamCsharp;

if (args.Length == 0)
{
    Console.WriteLine("Usage:");
    Console.WriteLine("  aam-csharp-console parse \"key = value\"");
    Console.WriteLine("  aam-csharp-console load <path-to-file.aam>");
    Environment.ExitCode = 1;
    return;
}

try
{
    switch (args[0].ToLowerInvariant())
    {
        case "parse" when args.Length >= 2:
            RunParse(args[1]);
            break;

        case "load" when args.Length >= 2:
            RunLoad(args[1]);
            break;

        default:
            Console.WriteLine("Invalid arguments.");
            Environment.ExitCode = 1;
            break;
    }
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
    Environment.ExitCode = 2;
}
catch (AamException ex)
{
    Console.WriteLine($"AAM error: {ex.Message}");
    Environment.ExitCode = 3;
}

static void RunParse(string content)
{
    using var doc = AamDocument.Parse(content);
    Console.WriteLine("Parsed successfully.");
}

static void RunLoad(string path)
{
    using var doc = AamDocument.Load(path);
    Console.WriteLine($"Loaded file: {path}");
}

