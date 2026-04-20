package com.epub.reader.ui.pagecurl

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.awaitTouchSlopOrCancellation
import androidx.compose.foundation.gestures.drag
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.util.VelocityTracker
import androidx.compose.ui.input.pointer.util.addPointerInputChange
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

/**
 * Gesture detector for the book spread curl.
 *
 * Forward drag: touch right half, drag left -> fold 1->0 (turn page).
 * Backward drag: drag right -> fold 0->1 (un-turn page).
 * Tap: no drag -> passed to [onTap].
 *
 * fold maps to foldX = spineX + halfWidth * fold, so:
 *   fold 1 = right edge, fold 0 = spine.
 *   fraction = (fingerX - spineX) / halfWidth
 */
internal fun Modifier.bookSpreadDragGesture(
    state: BookSpreadState,
    enabledForward: Boolean,
    enabledBackward: Boolean,
    scope: CoroutineScope,
    onForward: () -> Unit,
    onBackward: () -> Unit,
    onTap: ((position: Offset) -> Unit)? = null,
): Modifier = pointerInput(state.internalState, enabledForward, enabledBackward) {
    val w = size.width.toFloat()
    val spineX = w / 2f
    val halfW = w / 2f

    fun xToFold(x: Float) = (x / w).coerceIn(0f, 1f)

    awaitEachGesture {
        val velocityTracker = VelocityTracker()
        val down = awaitFirstDown(requireUnconsumed = false)
        velocityTracker.addPointerInputChange(down)

        val drag = awaitTouchSlopOrCancellation(down.id) { change, _ ->
            change.consume()
        }

        if (drag == null) {
            onTap?.invoke(down.position)
            return@awaitEachGesture
        }
        
        velocityTracker.addPointerInputChange(drag)

        val dragDelta = drag.position.x - down.position.x
        val isForwardDrag = dragDelta < 0 && enabledForward && down.position.x > spineX
        val isBackwardDrag = dragDelta > 0 && enabledBackward && down.position.x < spineX

        if (!isForwardDrag && !isBackwardDrag) return@awaitEachGesture

        val s = state.internalState ?: return@awaitEachGesture
        s.animateJob?.cancel()
        s.animateJob = null
        s.direction = if (isForwardDrag) SpreadDirection.FORWARD else SpreadDirection.BACKWARD

        var currentFingerFold = if (isForwardDrag) xToFold(drag.position.x) else 1f - xToFold(drag.position.x)
        val initialPageFold = 1f
        val catchupOffset = Animatable(initialPageFold - currentFingerFold)

        scope.launch { s.fold.snapTo(initialPageFold) }

        val catchupJob = scope.launch {
            catchupOffset.animateTo(0f, animationSpec = tween(150)) {
                val currentOffset = catchupOffset.value
                scope.launch { s.fold.snapTo((currentFingerFold + currentOffset).coerceIn(0f, 1f)) }
            }
        }

        drag(drag.id) { change ->
            velocityTracker.addPointerInputChange(change)
            currentFingerFold = if (isForwardDrag) xToFold(change.position.x) else 1f - xToFold(change.position.x)
            scope.launch { s.fold.snapTo((currentFingerFold + catchupOffset.value).coerceIn(0f, 1f)) }
            change.consume()
        }

        catchupJob.cancel()

        val velocity = velocityTracker.calculateVelocity().x
        val endFold = s.fold.value
        val flingThreshold = 400f // 像素/秒的滑动阈值

        scope.launch {
            try {
                if (isForwardDrag) {
                    val isFlingLeft = velocity < -flingThreshold
                    if (endFold < 0.5f || isFlingLeft) {
                        s.fold.animateTo(0f)
                        onForward()
                    } else {
                        s.fold.animateTo(1f)
                    }
                } else {
                    val isFlingRight = velocity > flingThreshold
                    if (endFold < 0.5f || isFlingRight) {
                        s.fold.animateTo(0f)
                        onBackward()
                    } else {
                        s.fold.animateTo(1f)
                    }
                }
            } finally {
                s.reset()
            }
        }
    }
}
