using System;

namespace AamCsharp;

/// <summary>
/// Represents an error returned by the native AAM engine.
/// </summary>
public sealed class AamException : Exception
{
    /// <summary>
    /// Initializes a new instance of the <see cref="AamException"/> class.
    /// </summary>
    /// <param name="message">Error message returned by the native layer.</param>
    public AamException(string message) : base(message)
    {
    }
}


