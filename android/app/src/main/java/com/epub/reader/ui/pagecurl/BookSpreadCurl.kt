package com.epub.reader.ui.pagecurl

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.drawscope.clipRect

/**
 * Book-spread page curl composable.
 *
 * 4 layers during animation (bottom to top):
 *   1. Left half of turning spread (static left page)
 *   2. Right half of bottom spread (revealed/covered next page)
 *   3. Back-flap: bottom spread clipped to back region (covers left page with next content)
 *   4. Front of turning spread clipped to [spineX, foldX] + shadow
 *
 * FORWARD: current right page curls toward spine, revealing next spread.
 *   - turningIndex = cur, bottomIndex = cur + 1
 *   - Back-flap shows next spread's left page covering the current left page.
 *
 * BACKWARD: previous right page uncurls from spine, covering current spread.
 *   - turningIndex = cur - 1, bottomIndex = cur
 *   - Back-flap shows current spread's left page (initially covering, then receding).
 */
@Composable
fun BookSpreadCurl(
    count: Int,
    state: BookSpreadState = rememberBookSpreadState(),
    modifier: Modifier = Modifier,
    backPageColor: Color = Color.White,
    backPageContentAlpha: Float = 0.3f,
    shadowAlpha: Float = 0.35f,
    onCustomTap: ((size: Size, position: Offset) -> Unit)? = null,
    content: @Composable (Int) -> Unit,
) {
    val scope = rememberCoroutineScope()

    BoxWithConstraints(modifier.clipToBounds()) {
        val widthPx = constraints.maxWidth.toFloat()
        val heightPx = constraints.maxHeight.toFloat()
        state.setup(count, widthPx, heightPx)

        val s = state.internalState ?: return@BoxWithConstraints
        val cur by rememberUpdatedState(state.current)
        val hasPrev = cur > 0
        val hasNext = cur < state.max - 1

        Box(
            modifier = Modifier
                .fillMaxSize()
                .bookSpreadDragGesture(
                    state = state,
                    enabledForward = hasNext,
                    enabledBackward = hasPrev,
                    scope = scope,
                    onForward = {
                        state.current = (state.current + 1).coerceAtMost(state.max - 1)
                    },
                    onBackward = {
                        state.current = (state.current - 1).coerceAtLeast(0)
                    },
                    onTap = onCustomTap?.let { handler ->
                        { position -> handler(Size(widthPx, heightPx), position) }
                    },
                )
        ) {
            val direction = s.direction
            val fold = s.fold.value
            val isAnimating = direction != SpreadDirection.NONE
            val isMirrored = direction == SpreadDirection.BACKWARD

            val mirrorModifier = if (isMirrored) {
                Modifier.graphicsLayer {
                    scaleX = -1f
                    transformOrigin = TransformOrigin(0.5f, 0.5f)
                }
            } else Modifier

            val renderContentWithMirror: @Composable (Int) -> Unit = { index ->
                Box(Modifier.fillMaxSize().then(mirrorModifier)) {
                    content(index)
                }
            }

            if (isAnimating) {
                // BACKWARD uses the exact same turning logic (grab right -> flip left) 
                // but renders everything perfectly horizontally mirrored!
                val turningIndex = cur
                val bottomIndex = when (direction) {
                    SpreadDirection.FORWARD -> cur + 1
                    SpreadDirection.BACKWARD -> cur - 1
                    else -> cur
                }

                Box(Modifier.fillMaxSize().then(mirrorModifier)) {
                    // Layer 1 (bottom): Left half of turning spread
                    if (turningIndex in 0 until state.max) {
                        Box(Modifier.fillMaxSize().drawWithContent {
                            val spineX = size.width / 2f
                            clipRect(0f, 0f, spineX, size.height) {
                                this@drawWithContent.drawContent()
                            }
                        }) {
                            renderContentWithMirror(turningIndex)
                        }
                    }

                    // Layer 2: Right half of bottom spread (revealed underneath)
                    if (bottomIndex in 0 until state.max) {
                        Box(Modifier.fillMaxSize().drawWithContent {
                            val spineX = size.width / 2f
                            clipRect(spineX, 0f, size.width, size.height) {
                                this@drawWithContent.drawContent()
                            }
                        }) {
                            renderContentWithMirror(bottomIndex)
                        }
                    }

                    // Layer 3: Front of turning page [spineX, foldX] (flat portion)
                    if (turningIndex in 0 until state.max) {
                        Box(Modifier.fillMaxSize().drawBookSpreadFront(fold)) {
                            renderContentWithMirror(turningIndex)
                        }
                    }

                    // Layer 4: Back-flap [backLeft, foldX] (folded-over portion ON TOP of front)
                    if (bottomIndex in 0 until state.max) {
                        Box(Modifier.fillMaxSize().drawBookSpreadBackFlap(fold)) {
                            renderContentWithMirror(bottomIndex)
                        }
                    }

                    // Layer 5 (top): Shadow overlay
                    Box(Modifier.fillMaxSize().drawBookSpreadShadow(fold, shadowAlpha)) {}
                }
            } else {
                // Idle: full current spread
                content(cur)
            }
        }
    }
}
