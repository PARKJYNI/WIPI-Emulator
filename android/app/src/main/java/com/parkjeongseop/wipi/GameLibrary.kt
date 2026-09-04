// 게임 라이브러리 — 임포트한 게임을 filesDir/games/<UUID>/에 영구 저장하고 관리.
// 표지·게임명은 패키지(zip) 안의 big.icon/__adf__에서 뽑아 캐시한다 (iOS GameLibrary와 대칭).

package com.parkjeongseop.wipi

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import android.provider.OpenableColumns
import org.json.JSONObject
import java.io.ByteArrayInputStream
import java.io.File
import java.nio.charset.Charset
import java.util.UUID
import java.util.zip.ZipInputStream

data class GameEntry(
    val id: String,
    val name: String,
    val cover: Bitmap?,
    val gameFile: File,
    val filename: String,
    val dataDir: File,
)

class GameLibrary(context: Context) {
    private val root = File(context.filesDir, "games").apply { mkdirs() }
    private val contentResolver = context.contentResolver
    private val eucKr = Charset.forName("EUC-KR")

    fun list(): List<GameEntry> =
        (root.listFiles() ?: emptyArray())
            .filter { it.isDirectory }
            .mapNotNull { load(it) }
            .sortedBy { it.name }

    private fun load(dir: File): GameEntry? {
        val metaFile = File(dir, "meta.json")
        if (!metaFile.exists()) return null
        return try {
            val meta = JSONObject(metaFile.readText())
            val filename = meta.getString("filename")
            val gameFile = File(dir, filename)
            if (!gameFile.exists()) return null
            val cover = File(dir, "cover.png").takeIf { it.exists() }?.let { BitmapFactory.decodeFile(it.path) }
            GameEntry(
                id = dir.name,
                name = meta.getString("name"),
                cover = cover,
                gameFile = gameFile,
                filename = filename,
                dataDir = File(dir, "data"),
            )
        } catch (e: Exception) {
            null
        }
    }

    fun importGame(uri: Uri): GameEntry? {
        val filename = queryDisplayName(uri) ?: uri.lastPathSegment ?: "game.zip"
        val bytes = contentResolver.openInputStream(uri)?.use { it.readBytes() } ?: return null

        val id = UUID.randomUUID().toString()
        val dir = File(root, id).apply { mkdirs() }
        File(dir, filename).writeBytes(bytes)
        WipiNative.nativeGameIcon(bytes)?.let { File(dir, "cover.png").writeBytes(it) }

        val name = WipiNative.nativeGameName(bytes)
            ?.toString(eucKr)
            ?: filename.substringBeforeLast('.')

        // KTF/WIPI 추가 다운로드 데이터 sideload.
        // wie의 Filesystem은 <dataDir>/<AID>/<게임이 요청한 경로> 를 실제 파일로 사용한다.
        // 패키지 ZIP 루트의 *.dat / list 파일을 해당 위치에 선주입한다.
        sideloadKtfDownloadData(bytes, File(dir, "data"))

        File(dir, "meta.json").writeText(
            JSONObject().put("name", name).put("filename", filename).toString()
        )

        return load(dir)
    }

    private fun sideloadKtfDownloadData(packageBytes: ByteArray, dataDir: File) {
        val entries = linkedMapOf<String, ByteArray>()
        var aid: String? = null

        ZipInputStream(ByteArrayInputStream(packageBytes)).use { zip ->
            while (true) {
                val entry = zip.nextEntry ?: break
                if (!entry.isDirectory) {
                    val entryName = entry.name.replace('\\', '/')
                    val baseName = entryName.substringAfterLast('/')
                    val data = zip.readBytes()

                    if (entryName == "__adf__" || baseName == "__adf__") {
                        val adf = data.toString(eucKr)
                        aid = adf.lineSequence()
                            .firstOrNull { it.startsWith("AID:") }
                            ?.substringAfter("AID:")
                            ?.trim()
                            ?.takeIf { it.isNotEmpty() }
                    }

                    // 추가 다운로드 원본은 패키지 최상위의 .dat / list 형태로 보존되는 경우가 있다.
                    if (!entryName.contains('/') && (baseName.endsWith(".dat", ignoreCase = true) || baseName == "list")) {
                        entries[baseName] = data
                    }
                }
                zip.closeEntry()
            }
        }

        val appId = aid ?: return
        if (entries.isEmpty()) return

        val appDataDir = File(dataDir, appId).apply { mkdirs() }
        for ((name, data) in entries) {
            File(appDataDir, name).writeBytes(data)
        }
    }

    fun delete(entry: GameEntry) {
        File(root, entry.id).deleteRecursively()
    }

    private fun queryDisplayName(uri: Uri): String? =
        contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) cursor.getString(0) else null
        }
}
