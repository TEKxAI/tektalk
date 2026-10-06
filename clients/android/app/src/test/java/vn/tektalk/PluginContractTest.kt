package vn.tektalk

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import vn.tektalk.plugins.PluginLifecycle
import vn.tektalk.plugins.PluginState

class PluginContractTest {
    @Test fun healthyInstallPromotesAtomically() {
        val lifecycle = PluginLifecycle()
        listOf(PluginState.DOWNLOADED, PluginState.VERIFIED, PluginState.STAGED, PluginState.ACTIVE).forEach(lifecycle::transition)
        assertEquals(PluginState.ACTIVE, lifecycle.state)
    }

    @Test fun invalidPromotionIsRejected() {
        assertThrows(IllegalArgumentException::class.java) { PluginLifecycle().transition(PluginState.ACTIVE) }
    }
}
