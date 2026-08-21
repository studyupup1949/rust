using AamCsharp;

// Example 1: Basic parsing
Console.WriteLine("=== Example 1: Basic Parsing ===");
var basicContent = @"
# Server configuration
host = localhost
port = 8080
debug = true
";

try
{
    using var doc = AamDocument.Parse(basicContent);
    Console.WriteLine($"Host: {doc.Get("host")}");
    Console.WriteLine($"Port: {doc.Get("port")}");
    Console.WriteLine($"Debug: {doc.Get("debug")}");
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
}

// Example 2: Pattern search
Console.WriteLine("\n=== Example 2: Deep Search by Key Pattern ===");
var baseConfig = @"
database_host = localhost
database_port = 5432
cache = redis
";

try
{
    using var doc = AamDocument.Parse(baseConfig);
    var dbEntries = doc.DeepSearch("database");
    foreach (var entry in dbEntries)
    {
        Console.WriteLine($"  {entry.Key}: {entry.Value}");
    }
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
}

// Example 3: Reverse search
Console.WriteLine("\n=== Example 3: Reverse Search ===");
var registryContent = @"
primary_db = postgresql
backup_db = mysql
cache_layer = redis
message_queue = rabbitmq
";

try
{
    using var doc = AamDocument.Parse(registryContent);
    var keys = doc.ReverseSearch("redis");
    if (keys.Length > 0)
    {
        Console.WriteLine($"Found keys for 'redis': {string.Join(", ", keys)}");
    }
    else
    {
        Console.WriteLine("Key not found");
    }
}
catch (DllNotFoundException ex)
{
    Console.WriteLine($"Native library not found: {ex.Message}");
}

Console.WriteLine("\nExamples completed!");

