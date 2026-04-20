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
internal fun Modifier.dragGesture(
    dragInteraction: PageCurlConfig.GestureDragInteraction,
    state: PageCurlState.InternalState,
    enabledForward: Boolean,
    enabledBackward: Boolean,
    scope: CoroutineScope,
    onChange: (Int) -> Unit
): Modifier = this.composed {
    val isEnabledForward = rememberUpdatedState(enabledForward)
    val isEnabledBackward = rememberUpdatedState(enabledBackward)

    pointerInput(state) {
        val forwardTargetRect by lazy { dragInteraction.forward.target.multiply(size) }
        val backwardTargetRect by lazy { dragInteraction.backward.target.multiply(size) }

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
            isDragSucceed = { start, end -> end.x < start.x },
            onChange = { onChange(+1) },
            edgeCreatorOverride = if (isBookSpread) NewEdgeCreator.BookSpread(baseCreator, spineX) else null,
        )
        val backwardConfig = DragConfig(
            edge = state.backward,
            start = if (isBookSpread) state.rightEdge else state.leftEdge,
            end = if (isBookSpread) state.spineEdge else state.rightEdge,
            isEnabled = { isEnabledBackward.value },
            isDragSucceed = { start, end -> end.x > start.x },
            onChange = { onChange(-1) },
            edgeCreatorOverride = if (isBookSpread) NewEdgeCreator.BookSpread(baseCreator, spineX, mirrored = true) else null,
        )

        detectCurlGestures(
            scope = scope,
            newEdgeCreator = baseCreator,
            getConfig = { start, end ->
                val config = if (forwardTargetRect.contains(start) && end.x < start.x) {
                    forwardConfig
                } else if (backwardTargetRect.contains(start) && end.x > start.x) {
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
