package com.zhongbai233.epub.reader.ui.reader

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.zhongbai233.epub.reader.i18n.I18n

@Composable
fun ContributeDialog(
    show: Boolean,
    sampleCount: Int,
    samples: String,
    githubUsername: String?,
    githubUserCode: String?,
    githubVerificationUri: String?,
    githubAuthPolling: Boolean,
    inProgress: Boolean,
    status: String,
    prUrl: String?,
    onPrepare: () -> Unit,
    onLogin: () -> Unit,
    onSubmit: () -> Unit,
    onDismiss: () -> Unit,
) {
    if (!show) return

    // Prepare samples when dialog opens
    LaunchedEffect(show) {
        if (show) onPrepare()
    }

    val context = LocalContext.current
    val clipboardManager = LocalClipboardManager.current

    AlertDialog(
        onDismissRequest = { if (!inProgress) onDismiss() },
        confirmButton = {
            if (prUrl != null) {
                // Success state — show close + open PR
                Row {
                    TextButton(onClick = {
                        context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(prUrl)))
                    }) {
                        Text(I18n.t("csc.contribute_success").replace("{}", "PR"))
                    }
                    Spacer(Modifier.width(8.dp))
                    Button(onClick = onDismiss) {
                        Text(I18n.t("csc.contribute_close"))
                    }
                }
            } else {
                Button(
                    onClick = onSubmit,
                    enabled = !inProgress && githubToken(githubUsername) && sampleCount > 0
                ) {
                    if (inProgress) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(16.dp),
                            strokeWidth = 2.dp,
                            color = MaterialTheme.colorScheme.onPrimary
                        )
                        Spacer(Modifier.width(8.dp))
                        Text(I18n.t("csc.contribute_submitting"))
                    } else {
                        Text(I18n.t("csc.contribute_submit"))
                    }
                }
            }
        },
        dismissButton = {
            if (!inProgress && prUrl == null) {
                TextButton(onClick = onDismiss) {
                    Text(I18n.t("csc.contribute_later"))
                }
            }
        },
        title = { Text(I18n.t("csc.contribute_title")) },
        text = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                // Description
                Text(
                    text = I18n.t("csc.contribute_desc"),
                    style = MaterialTheme.typography.bodyMedium
                )

                // Sample count
                Text(
                    text = I18n.t("csc.contribute_sample_count").replace("{}", sampleCount.toString()),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.primary
                )

                // Preview
                if (samples.isNotBlank()) {
                    Text(
                        text = I18n.t("csc.contribute_preview"),
                        style = MaterialTheme.typography.labelMedium
                    )
                    Surface(
                        color = MaterialTheme.colorScheme.surfaceVariant,
                        shape = MaterialTheme.shapes.small,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Text(
                            text = samples.lines().take(5).joinToString("\n"),
                            modifier = Modifier.padding(8.dp),
                            style = MaterialTheme.typography.bodySmall.copy(
                                fontFamily = FontFamily.Monospace,
                                fontSize = 11.sp
                            ),
                            maxLines = 5,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }

                HorizontalDivider()

                // GitHub login section
                if (githubUsername != null) {
                    Text(
                        text = I18n.t("csc.contribute_logged_in").replace("{}", githubUsername),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.primary
                    )
                } else if (githubUserCode != null && githubVerificationUri != null) {
                    // Show device code for user to enter
                    Text(
                        text = "请在浏览器中输入以下代码：",
                        style = MaterialTheme.typography.bodySmall
                    )
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Text(
                            text = githubUserCode,
                            style = MaterialTheme.typography.headlineSmall.copy(
                                fontFamily = FontFamily.Monospace
                            )
                        )
                        TextButton(onClick = {
                            clipboardManager.setText(AnnotatedString(githubUserCode))
                        }) {
                            Text("复制")
                        }
                    }
                    TextButton(onClick = {
                        context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(githubVerificationUri)))
                    }) {
                        Text("打开 GitHub 验证页面")
                    }
                    if (githubAuthPolling) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            CircularProgressIndicator(modifier = Modifier.size(16.dp), strokeWidth = 2.dp)
                            Text("等待授权...", style = MaterialTheme.typography.bodySmall)
                        }
                    }
                } else {
                    Text(
                        text = I18n.t("csc.contribute_need_login"),
                        style = MaterialTheme.typography.bodySmall
                    )
                    OutlinedButton(onClick = onLogin) {
                        Text(I18n.t("csc.login_github"))
                    }
                }

                // Status / Error
                if (status.isNotBlank()) {
                    Text(
                        text = status,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error
                    )
                }

                // Success PR URL
                if (prUrl != null) {
                    Text(
                        text = I18n.t("csc.contribute_success").replace("{}", prUrl),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.primary
                    )
                }
            }
        }
    )
}

private fun githubToken(username: String?): Boolean = username != null
