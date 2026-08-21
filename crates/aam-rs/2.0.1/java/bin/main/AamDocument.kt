package com.rustgames.aam

import java.io.File
import java.io.InputStream
import java.lang.ref.Cleaner
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.atomic.AtomicLong

/**
 * High-level JVM wrapper around the native aam-rs parser.
 */
class AamDocument private constructor(private var nativePtr: Long) : AutoCloseable {
    private val cleanable = CLEANER.register(this, NativeResource(nativePtr))

    private class NativeResource(ptr: Long) : Runnable {
        private val ptr = AtomicLong(ptr)
        override fun run() {
            val value = ptr.getAndSet(0L)
            if (value != 0L) {
                AAM.destroy(value)
            }
        }
    }

    companion object {
        private val CLEANER: Cleaner = Cleaner.create()

        init {
            loadNativeLibrary()
        }

        private fun loadNativeLibrary() {
            val osName = System.getProperty("os.name").lowercase()
            val osArch = System.getProperty("os.arch").lowercase()

            val osPrefix = when {
                osName.contains("win") -> "windows"
                osName.contains("mac") -> "macos"
                else -> "linux"
            }

            val archPrefix = when {
                osArch.contains("aarch64") || osArch.contains("arm64") -> "aarch64"
                else -> "x86_64"
            }

            val extension = when (osPrefix) {
                "windows" -> ".dll"
                "macos" -> ".dylib"
                else -> ".so"
            }

            val libName = if (osPrefix == "windows") "aam_rs$extension" else "libaam_rs$extension"
            val resourcePath = "/natives/$osPrefix-$archPrefix/$libName"

            val inputStream: InputStream? = AamDocument::class.java.getResourceAsStream(resourcePath)
                ?: throw UnsupportedOperationException("Native library not found in JAR at $resourcePath")

            val tempFile = File.createTempFile("libaam_rs_", extension)
            tempFile.deleteOnExit()

            inputStream!!.use { input ->
                Files.copy(input, tempFile.toPath(), StandardCopyOption.REPLACE_EXISTING)
            }

            System.load(tempFile.absolutePath)
        }

        @JvmStatic
        fun parse(content: String): AamDocument {
            val ptr = AAM.parse(content)
            if (ptr == 0L) throw IllegalStateException("Failed to parse AAM content")
            return AamDocument(ptr)
        }

        @JvmStatic
        fun load(path: String): AamDocument {
            val ptr = AAM.load(path)
            if (ptr == 0L) throw IllegalStateException("Failed to load AAM file: $path")
            return AamDocument(ptr)
        }
    }

    /**
     * Completely reloads the document state with new content.
     * Overwrites all existing data in memory.
     */
    fun reload(content: String) {
        AAM.reload(checkPtr(), content)
    }

    fun get(key: String): String? = AAM.get(checkPtr(), key)

    fun deepSearch(pattern: String): Map<String, String> {
        return AAM.deepSearch(checkPtr(), pattern) ?: emptyMap()
    }

    fun reverseSearch(value: String): List<String> {
        return AAM.reverseSearch(checkPtr(), value)?.toList() ?: emptyList()
    }

    fun schemaNames(): List<String> {
        return AAM.schemaNames(checkPtr())?.toList() ?: emptyList()
    }

    fun typeNames(): List<String> {
        return AAM.typeNames(checkPtr())?.toList() ?: emptyList()
    }

    override fun close() {
        if (nativePtr != 0L) {
            cleanable.clean()
            nativePtr = 0L
        }
    }

    private fun checkPtr(): Long {
        if (nativePtr == 0L) throw IllegalStateException("AamDocument is closed")
        return nativePtr
    }
}

/**
 * Helper for JNI calls to the native library. All native interactions are funneled through this object.
 */
private object AAM {
    @JvmStatic external fun new(): Long
    @JvmStatic external fun parse(content: String): Long
    @JvmStatic external fun load(path: String): Long
    @JvmStatic external fun reload(ptr: Long, content: String)
    @JvmStatic external fun destroy(ptr: Long)

    @JvmStatic external fun get(ptr: Long, key: String): String?
    @JvmStatic external fun deepSearch(ptr: Long, pattern: String): HashMap<String, String>?
    @JvmStatic external fun reverseSearch(ptr: Long, value: String): Array<String>?

    @JvmStatic external fun schemaNames(ptr: Long): Array<String>?
    @JvmStatic external fun typeNames(ptr: Long): Array<String>?
}