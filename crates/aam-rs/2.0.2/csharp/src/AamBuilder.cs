using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

namespace AamCsharp;

public readonly struct SchemaField
{
    public string Name { get; }
    public string TypeName { get; }
    public bool Optional { get; }

    private SchemaField(string name, string typeName, bool optional)
    {
        Name = name;
        TypeName = typeName;
        Optional = optional;
    }

    public static SchemaField Required(string name, string typeName) => new(name, typeName, false);
    public static SchemaField OptionalField(string name, string typeName) => new(name, typeName, true);

    public string ToAam() => Optional ? $"{Name}*: {TypeName}" : $"{Name}: {TypeName}";
}

public sealed class AamBuilder
{
    private readonly StringBuilder _buffer;

    public AamBuilder() : this(0)
    {
    }

    public AamBuilder(int capacity)
    {
        _buffer = capacity > 0 ? new StringBuilder(capacity) : new StringBuilder();
    }

    public AamBuilder AddLine(string key, string value)
    {
        PushSeparator();
        _buffer.Append(key).Append(" = ").Append(value);
        return this;
    }

    public AamBuilder Comment(string text)
    {
        PushSeparator();
        _buffer.Append("# ").Append(text);
        return this;
    }

    public AamBuilder Schema(string name, IEnumerable<SchemaField> fields)
    {
        PushSeparator();
        _buffer.Append("@schema ").Append(name).Append(" { ");

        var first = true;
        foreach (var field in fields)
        {
            if (!first)
            {
                _buffer.Append(", ");
            }

            _buffer.Append(field.ToAam());
            first = false;
        }

        _buffer.Append(" }");
        return this;
    }

    public AamBuilder SchemaMultiline(string name, IEnumerable<SchemaField> fields)
    {
        PushSeparator();
        _buffer.Append("@schema ").Append(name).Append(" {");
        foreach (var field in fields)
        {
            _buffer.AppendLine();
            _buffer.Append("    ").Append(field.ToAam());
        }

        _buffer.AppendLine();
        _buffer.Append('}');
        return this;
    }

    public AamBuilder Derive(string path, IEnumerable<string> schemas)
    {
        PushSeparator();
        _buffer.Append("@derive ").Append(path);
        foreach (var schema in schemas)
        {
            _buffer.Append("::").Append(schema);
        }

        return this;
    }

    public AamBuilder Import(string path)
    {
        PushSeparator();
        _buffer.Append("@import ").Append(path);
        return this;
    }

    public AamBuilder TypeAlias(string alias, string typeName)
    {
        PushSeparator();
        _buffer.Append("@type ").Append(alias).Append(" = ").Append(typeName);
        return this;
    }

    public string Build() => _buffer.ToString();

    public void ToFile(string path)
    {
        File.WriteAllText(path, _buffer.ToString());
    }

    public override string ToString() => _buffer.ToString();

    private void PushSeparator()
    {
        if (_buffer.Length > 0)
        {
            _buffer.AppendLine();
        }
    }
}

