package com.epub.reader.ui.pagecurl

import androidx.compose.animation.core.Animatable
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

@Composable
fun rememberBookSpreadState(
    initialCurrent: Int = 0,
): BookSpreadState =
    rememberSaveable(
        initialCurrent,
        saver = Saver(
            save = { it.current },
            restore = { BookSpreadState(initialCurrent = it) }
        )
    ) {
        BookSpreadState(initialCurrent = initialCurrent)
    }

enum class SpreadDirection { NONE, FORWARD, BACKWARD }

/**
 * State for the book-spread page curl.
 *
 * Uses a single [fold] value + [direction] instead of dual folds:
 *   fold = 1.0  → page flat at right edge (idle)
 *   fold = 0.0  → page fully turned at center spine
 *
 * FORWARD:  fold animates 1 → 0   (right page curls toward spine)
 * BACKWARD: fold animates 0 → 1   (previous page uncurls from spine)
 */
class BookSpreadState(
    initialMax: Int = 0,
    initialCurrent: Int = 0,
) {
    var current: Int by mutableStateOf(initialCurrent)
        internal set

    internal var max: Int = initialMax
        private set

    internal var internalState: InternalState? by mutableStateOf(null)
        private set

    internal fun setup(count: Int, widthPx: Float, heightPx: Float) {
        max = count
        if (current >= count) current = (count - 1).coerceAtLeast(0)
        val cur = internalState
        if (cur != null && cur.widthPx == widthPx && cur.heightPx == heightPx) return
        internalState = InternalState(widthPx, heightPx)
    }

    suspend fun snapTo(value: Int) {
        current = value.coerceIn(0, (max - 1).coerceAtLeast(0))
        internalState?.reset()
    }

    /** Animate forward: right page curls toward spine, then current++ */
    suspend fun next() {
        val s = internalState ?: return
        val target = current + 1
        if (target >= max) return
        s.animateJob?.cancel()
        coroutineScope {
            s.animateJob = launch {
                try {
                    s.direction = SpreadDirection.FORWARD
                    s.fold.snapTo(1f)
                    s.fold.animateTo(0f)
                } finally {
                    withContext(NonCancellable) {
                        current = target
                        s.reset()
                    }
                }
            }
        }
    }

    /** Animate backward: current left page curls toward spine, then current-- */
    suspend fun prev() {
        val s = internalState ?: return
        val target = current - 1
        if (target < 0) return
        s.animateJob?.cancel()
        coroutineScope {
            s.animateJob = launch {
                try {
                    s.direction = SpreadDirection.BACKWARD
                    s.fold.snapTo(1f)
                    s.fold.animateTo(0f)
                } finally {
                    withContext(NonCancellable) {
                        current = target
                        s.reset()
                    }
                }
            }
        }
    }

    internal class InternalState(
        val widthPx: Float,
        val heightPx: Float,
    ) {
        val fold = Animatable(1f)
        var direction: SpreadDirection by mutableStateOf(SpreadDirection.NONE)
        var animateJob: Job? = null

        suspend fun reset() {
            fold.snapTo(1f)
            direction = SpreadDirection.NONE
        }
    }
}
