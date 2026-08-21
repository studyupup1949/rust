using AamCsharp;
using Xunit;

namespace AamCsharp.Tests;

public sealed class AamDocumentTests
{
    private static void SkipIfNativeMissing(Action assertion)
    {
        try
        {
            assertion();
        }
        catch (DllNotFoundException)
        {
            // CI environments without native artifacts should not fail this managed test suite.
        }
    }

    [Fact]
    public void ParseAndGet_ReturnsValue_WhenNativeLibraryIsAvailable()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("host = localhost\nport = 8080");
            Assert.Equal("localhost", doc.Get("host"));
        });
    }

    [Fact]
    public void Parse_MultipleKeys()
    {
        SkipIfNativeMissing(() =>
        {
            const string content = @"
name = MyApp
version = 1.0.0
debug = true
";
            using var doc = AamDocument.Parse(content);
            Assert.Equal("MyApp", doc.Get("name"));
            Assert.Equal("1.0.0", doc.Get("version"));
            Assert.Equal("true", doc.Get("debug"));
        });
    }

    [Fact]
    public void Parse_WithComments()
    {
        SkipIfNativeMissing(() =>
        {
            const string content = @"
# This is a comment
host = localhost
# Another comment
port = 8080
";
            using var doc = AamDocument.Parse(content);
            Assert.Equal("localhost", doc.Get("host"));
            Assert.Equal("8080", doc.Get("port"));
        });
    }


    [Fact]
    public void ReverseSearch_FindsKeysByValue()
    {
        SkipIfNativeMissing(() =>
        {
            const string content = @"
database = postgres
cache = redis
messaging = rabbitmq
";
            using var doc = AamDocument.Parse(content);
            var result = doc.ReverseSearch("postgres");
            Assert.Contains("database", result);
        });
    }

    [Fact]
    public void ParseEmptyDocument()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("");
            Assert.Null(doc.Get("nonexistent"));
        });
    }

    [Fact]
    public void ParseWithWhitespace()
    {
        SkipIfNativeMissing(() =>
        {
            const string content = @"
name   =   MyApp
port   =   8080
";
            using var doc = AamDocument.Parse(content);
            Assert.Equal("MyApp", doc.Get("name"));
            Assert.Equal("8080", doc.Get("port"));
        });
    }

    [Fact]
    public void Find_PerformsKeyAndValueLookup()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("username = admin");
            Assert.Equal("admin", doc.Find("username")["username"]);
            Assert.Equal("admin", doc.Find("admin")["username"]);
        });
    }

    [Fact]
    public void DeepSearch_ReturnsMatchingPairs()
    {
        SkipIfNativeMissing(() =>
        {
            using var doc = AamDocument.Parse("root_path = srv\ncurrent_path = root_path\nmode = active");
            var result = doc.DeepSearch("path");
            Assert.Equal("srv", result["root_path"]);
            Assert.Equal("root_path", result["current_path"]);
        });
    }

    [Fact]
    public void Parse_InvalidContentThrowsAamException()
    {
        SkipIfNativeMissing(() =>
        {
            try
            {
                using var _ = AamDocument.Parse("invalid_line_without_equals");
                Assert.Fail("Expected AamException for invalid content");
            }
            catch (AamException)
            {
                // Expected.
            }
        });
    }
}



