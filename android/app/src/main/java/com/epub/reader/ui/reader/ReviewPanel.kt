package com.zhongbai233.epub.reader.ui.reader

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.zhongbai233.epub.reader.i18n.I18n
import com.zhongbai233.epub.reader.model.ContentBlock

/** Parsed review card data */
private data class ReviewCard(
    val index: Int,
    val content: String,
    val author: String,
    val timestamp: String,
    val likes: Int
)

/**
 * Try to parse a review paragraph into structured card data.
 * Expected format: "1. 【内容】 作者：xxx | 时间：xxx | 赞：52"
 */
private fun parseReviewCard(text: String): ReviewCard? {
    val trimmed = text.trim()

    // Extract index: "1. ..."
    val dotPos = trimmed.indexOf('.')
    if (dotPos <= 0) return null
    val index = trimmed.substring(0, dotPos).trim().toIntOrNull() ?: return null
    val rest = trimmed.substring(dotPos + 1).trim()

    // Split by " | " or " ｜ "
    val parts = rest.split("|", "｜")
    if (parts.size < 3) return null

    // Last part: "赞：52"
    val likesPart = parts.last().trim()
    val likesStr = likesPart.removePrefix("赞：").removePrefix("赞:").trim()
    val likes = likesStr.toIntOrNull() ?: return null

    // Second-to-last part: "时间：1770036499"
    val timePart = parts[parts.size - 2].trim()
    val timestamp = timePart.removePrefix("时间：").removePrefix("时间:").trim()

    // First part: "【内容】 作者：吃草莓布丁吗"
    val firstPart = parts[0].trim()
    val authorFull = "作者："
    val authorAscii = "作者:"
    val authorPos = firstPart.lastIndexOf(authorFull).takeIf { it >= 0 }
        ?: firstPart.lastIndexOf(authorAscii).takeIf { it >= 0 }
        ?: return null
    val authorMarkerLen = if (firstPart.contains(authorFull)) authorFull.length else authorAscii.length
    val content = firstPart.substring(0, authorPos).trim()
    val author = firstPart.substring(authorPos + authorMarkerLen).trim()

    return ReviewCard(index, content, author, timestamp, likes)
}

/**
 * 段评面板 — 底部弹层展示段评内容
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ReviewPanel(
    chapterTitle: String,
    blocks: List<ContentBlock>,
    anchorId: String? = null,
    fontSize: Float = 16f,
    onDismiss: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        dragHandle = { BottomSheetDefaults.DragHandle() }
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(0.80f)
                .padding(horizontal = 16.dp)
        ) {
            // Header
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    I18n.t("review.panel_title"),
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold
                )
                IconButton(onClick = onDismiss) {
                    Icon(
                        Icons.Default.Close,
                        contentDescription = I18n.t("dialog.close")
                    )
                }
            }

            Text(
                chapterTitle,
                fontSize = 13.sp,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            HorizontalDivider()
            Spacer(Modifier.height(8.dp))

            // Filter blocks by anchor if specified
            val filteredBlocks = remember(blocks, anchorId) {
                if (anchorId.isNullOrBlank()) blocks
                else {
                    blocks.filter { block ->
                        val blockAnchor = when (block) {
                            is ContentBlock.Heading -> block.anchor_id
                            is ContentBlock.Paragraph -> block.anchor_id
                            else -> null
                        }
                        blockAnchor != null && (
                            blockAnchor == anchorId ||
                            blockAnchor.contains(anchorId) ||
                            anchorId.contains(blockAnchor)
                        )
                    }
                }
            }

            // Content
            LazyColumn {
                items(filteredBlocks) { block ->
                    when (block) {
                        is ContentBlock.Heading -> {
                            val text = block.spans.joinToString("") { it.text }
                            val size = when (block.level) {
                                1 -> 18.sp
                                2 -> 16.sp
                                else -> 14.sp
                            }
                            Text(
                                text = text,
                                fontSize = size,
                                fontWeight = FontWeight.Bold,
                                modifier = Modifier.padding(vertical = 6.dp)
                            )
                        }
                        is ContentBlock.Paragraph -> {
                            val text = block.spans.joinToString("") { it.text }
                            if (text.isNotBlank()) {
                                val card = parseReviewCard(text)
                                if (card != null) {
                                    ReviewCardItem(card = card, fontSize = fontSize)
                                } else {
                                    Text(
                                        text = text,
                                        fontSize = fontSize.sp,
                                        modifier = Modifier.padding(vertical = 4.dp)
                                    )
                                }
                            }
                        }
                        is ContentBlock.Separator -> {
                            HorizontalDivider(
                                modifier = Modifier.padding(vertical = 8.dp),
                                color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f)
                            )
                        }
                        is ContentBlock.BlankLine -> {
                            Spacer(Modifier.height((fontSize * 0.5).dp))
                        }
                        else -> {}
                    }
                }
            }
        }
    }
}

@Composable
private fun ReviewCardItem(card: ReviewCard, fontSize: Float) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 6.dp),
        shape = RoundedCornerShape(12.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
        )
    ) {
        Column(
            modifier = Modifier.padding(12.dp)
        ) {
            // Content
            Text(
                text = card.content,
                fontSize = fontSize.sp,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.padding(bottom = 8.dp)
            )
            // Author / Time / Likes
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = card.author,
                    fontSize = 13.sp,
                    color = MaterialTheme.colorScheme.primary,
                    fontWeight = FontWeight.SemiBold
                )
                Text(
                    text = "${card.timestamp} · 赞 ${card.likes}",
                    fontSize = 12.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
                )
            }
        }
    }
}
