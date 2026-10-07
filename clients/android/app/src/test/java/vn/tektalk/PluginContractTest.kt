package vn.tektalk

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test
import vn.tektalk.plugins.PluginLifecycle
import vn.tektalk.plugins.PluginState
import vn.tektalk.plugins.PluginColorScheme
import vn.tektalk.plugins.PluginLayoutClass
import vn.tektalk.plugins.PluginUIEnvironment

class PluginContractTest {
    @Test fun uiEnvironmentSupportsTabletDarkAndLocale() {
        val environment = PluginUIEnvironment("vi-VN", PluginColorScheme.DARK, PluginLayoutClass.EXPANDED, 1.2f)
        assertEquals("vi-VN", environment.locale)
        assertEquals(PluginLayoutClass.EXPANDED, environment.layoutClass)
    }
    @Test fun healthyInstallPromotesAtomically() {
        val lifecycle = PluginLifecycle()
        listOf(PluginState.DOWNLOADED, PluginState.VERIFIED, PluginState.STAGED, PluginState.ACTIVE).forEach(lifecycle::transition)
        assertEquals(PluginState.ACTIVE, lifecycle.state)
    }

    @Test fun invalidPromotionIsRejected() {
        assertThrows(IllegalArgumentException::class.java) { PluginLifecycle().transition(PluginState.ACTIVE) }
    }
}
