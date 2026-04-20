package eu.wewox.pagecurl.page

import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.input.pointer.pointerInput
import eu.wewox.pagecurl.ExperimentalPageCurlApi
import eu.wewox.pagecurl.config.PageCurlConfig
import eu.wewox.pagecurl.config.PageCurlConfig.DragInteraction.PointerBehavior
import eu.wewox.pagecurl.utils.multiply
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

@ExperimentalPageCurlApi
internal fun Modifier.dragStartEnd(
    dragInteraction: PageCurlConfig.StartEndDragInteraction,
    state: PageCurlState.InternalState,
    enabledForward: Boolean,
    enabledBackward: Boolean,
    scope: CoroutineScope,
    onChange: (Int) -> Unit,
): Modifier = this.composed {
    val isEnabledForward = rememberUpdatedState(enabledForward)
    val isEnabledBackward = rememberUpdatedState(enabledBackward)

    pointerInput(state) {
        val forwardStartRect by lazy { dragInteraction.forward.start.multiply(size) }
        val forwardEndRect by lazy { dragInteraction.forward.end.multiply(size) }
        val backwardStartRect by lazy { dragInteraction.backward.start.multiply(size) }
        val backwardEndRect by lazy { dragInteraction.backward.end.multiply(size) }

        val isBookSpread = state.isBookSpread
        val spineX = size.width / 2f
        val baseCreator = when (dragInteraction.pointerBehavior) {
            PointerBehavior.Default -> NewEdgeCreator.Default()
            PointerBehavior.PageEdge -> NewEdgeCreator.PageEdge()
        }

        val forwardConfig = DragConfig(
            edge = state.forward,
            start = state.rightEdge,
            end = if (isBookSpread) state.spineEdge else state.leftEdge,
            isEnabled = { isEnabledForward.value },
            isDragSucceed = { _, end -> forwardEndRect.contains(end) },
            onChange = { onChange(+1) },
            edgeCreatorOverride = if (isBookSpread) NewEdgeCreator.BookSpread(baseCreator, spineX) else null,
        )
        val backwardConfig = DragConfig(
            edge = state.backward,
            start = if (isBookSpread) state.rightEdge else state.leftEdge,
            end = if (isBookSpread) state.spineEdge else state.rightEdge,
            isEnabled = { isEnabledBackward.value },
            isDragSucceed = { _, end -> backwardEndRect.contains(end) },
            onChange = { onChange(-1) },
            edgeCreatorOverride = if (isBookSpread) NewEdgeCreator.BookSpread(baseCreator, spineX, mirrored = true) else null,
        )

        detectCurlGestures(
            scope = scope,
            newEdgeCreator = baseCreator,
            getConfig = { start, _ ->
                val config = if (forwardStartRect.contains(start)) {
                    forwardConfig
                } else if (backwardStartRect.contains(start)) {
                    backwardConfig
                } else {
                    null
                }

                if (config != null) {
                    scope.launch {
                        state.animateJob?.cancel()
                        state.reset()
                    }
                }

                config
            },
        )
    }
}
