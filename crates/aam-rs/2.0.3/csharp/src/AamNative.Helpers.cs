using System;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace AamCsharp;

internal sealed unsafe class SafeAamHandle : SafeHandleZeroOrMinusOneIsInvalid
{
    public SafeAamHandle() : base(true)
    {
        SetHandle((IntPtr)AamNative.aam_new());
        if (IsInvalid)
        {
            throw new InvalidOperationException("Failed to allocate native AAM handle");
        }
    }

    protected override bool ReleaseHandle()
    {
        AamNative.aam_free((AamHandle*)handle);
        return true;
    }
}

internal static unsafe partial class AamNative
{
    internal static int aam_parse(SafeAamHandle handle, string content)
    {
        var utf8 = ToNullTerminatedUtf8(content);
        fixed (byte* ptr = utf8)
        {
            return aam_parse((AamHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static int aam_load(SafeAamHandle handle, string path)
    {
        var utf8 = ToNullTerminatedUtf8(path);
        fixed (byte* ptr = utf8)
        {
            return aam_load((AamHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_format(SafeAamHandle handle, string content)
    {
        var utf8 = ToNullTerminatedUtf8(content);
        fixed (byte* ptr = utf8)
        {
            return aam_format((AamHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_get(SafeAamHandle handle, string key)
    {
        var utf8 = ToNullTerminatedUtf8(key);
        fixed (byte* ptr = utf8)
        {
            return aam_get((AamHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_find(SafeAamHandle handle, string query)
    {
        var utf8 = ToNullTerminatedUtf8(query);
        fixed (byte* ptr = utf8)
        {
            return aam_find((AamHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_deep_search(SafeAamHandle handle, string pattern)
    {
        var utf8 = ToNullTerminatedUtf8(pattern);
        fixed (byte* ptr = utf8)
        {
            return aam_deep_search((AamHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_reverse_search(SafeAamHandle handle, string value)
    {
        var utf8 = ToNullTerminatedUtf8(value);
        fixed (byte* ptr = utf8)
        {
            return aam_reverse_search((AamHandle*)handle.DangerousGetHandle(), ptr);
        }
    }

    internal static byte* aam_schema_names(SafeAamHandle handle)
    {
        return aam_schema_names((AamHandle*)handle.DangerousGetHandle());
    }

    internal static byte* aam_type_names(SafeAamHandle handle)
    {
        return aam_type_names((AamHandle*)handle.DangerousGetHandle());
    }

    internal static byte* aam_last_error(SafeAamHandle handle)
    {
        return aam_last_error((AamHandle*)handle.DangerousGetHandle());
    }


    internal static string? BorrowUtf8String(byte* ptr)
    {
        if (ptr == null)
        {
            return null;
        }

        return Marshal.PtrToStringUTF8((IntPtr)ptr);
    }

    internal static string? TakeOwnedUtf8String(byte* ptr)
    {
        if (ptr == null)
        {
            return null;
        }

        try
        {
            return Marshal.PtrToStringUTF8((IntPtr)ptr);
        }
        finally
        {
            aam_string_free(ptr);
        }
    }

    private static byte[] ToNullTerminatedUtf8(string value)
    {
        var utf8 = Encoding.UTF8.GetBytes(value);
        var nullTerminated = new byte[utf8.Length + 1];
        Buffer.BlockCopy(utf8, 0, nullTerminated, 0, utf8.Length);
        return nullTerminated;
    }
}

