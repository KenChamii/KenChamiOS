package com.kenchami.kenchamios.ui

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.isToggleable
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onFirst
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.unit.dp
import com.kenchami.kenchamios.context.Rule
import com.kenchami.kenchamios.context.placeRule
import com.kenchami.kenchamios.context.wifiRule
import com.kenchami.kenchamios.ui.rules.ContextSettingsContent
import com.kenchami.kenchamios.ui.rules.ContextSettingsState
import com.kenchami.kenchamios.ui.rules.LEAVING_HOME_PRESET
import com.kenchami.kenchamios.ui.rules.NEW_RULE
import com.kenchami.kenchamios.ui.rules.PermissionSteps
import com.kenchami.kenchamios.ui.rules.RuleEditorContent
import com.kenchami.kenchamios.ui.rules.RuleForm
import com.kenchami.kenchamios.ui.rules.RulesContent
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.junit.Rule as JUnitRule

@RunWith(RobolectricTestRunner::class)
class RuleScreensTest {
    @get:JUnitRule
    val compose = createComposeRule()

    private val none = PaddingValues(0.dp)

    @Test
    fun theListShowsEachRuleWithItsSummaryAndSwitch() {
        val toggled = mutableListOf<Pair<String, Boolean>>()
        compose.setContent {
            RulesContent(listOf(placeRule("a"), wifiRule("w", enabled = false)), { rule: Rule, on -> toggled += rule.id to on }, {}, none)
        }

        compose.onNodeWithText("Place a").assertIsDisplayed()
        compose.onAllNodes(isToggleable()).onFirst().performClick()
        assertEquals(listOf("a" to false), toggled)
        compose.onNodeWithText("When you join a Wi-Fi network - Show tasks due today").assertIsDisplayed()
    }

    @Test
    fun anEmptyListPointsToTheLeavingHomePreset() {
        var opened: String? = null
        compose.setContent { RulesContent(emptyList(), { _, _ -> }, { opened = it }, none) }

        compose.onNodeWithText("No rules yet", substring = true).assertIsDisplayed()
        compose.onNodeWithText("Add the Leaving home checklist").performClick()

        assertEquals(LEAVING_HOME_PRESET, opened)
        compose.onNodeWithText("Add a rule").performClick()
        assertEquals(NEW_RULE, opened)
    }

    @Test
    fun tappingARuleOpensItsEditor() {
        var opened: String? = null
        compose.setContent { RulesContent(listOf(placeRule("a")), { _, _ -> }, { opened = it }, none) }

        compose.onNodeWithText("Place a").performClick()

        assertEquals("a", opened)
    }

    @Test
    fun theEditorShowsTheFieldsOfTheChosenKindAndReportsEdits() {
        var form = RuleForm()
        var saves = 0
        compose.setContent {
            RuleEditorContent(form, null, isNew = true, onChange = { change ->
                form = change(form)
            }, onSave = { saves++ }, onDelete = {}, padding = none)
        }

        compose.onNodeWithText("Latitude").assertIsDisplayed()
        compose.onNodeWithText("Name").performTextInput("Home")
        compose.onNodeWithText("Save").performScrollTo().performClick()

        assertEquals("Home", form.name)
        assertEquals(1, saves)
        compose.onNodeWithText("Delete").assertDoesNotExist()
    }

    @Test
    fun theEditorShowsTheValidationMessageAndOffersDeleteForExistingRules() {
        compose.setContent {
            RuleEditorContent(RuleForm(), "Give the rule a name.", isNew = false, onChange = {}, onSave = {}, onDelete = {}, padding = none)
        }

        compose.onNodeWithText("Give the rule a name.").performScrollTo().assertIsDisplayed()
        compose.onNodeWithText("Delete").performScrollTo().assertIsDisplayed()
    }

    @Test
    fun theSettingsExplainTheBackgroundStepAndOnlyOffersItAfterLocation() {
        var allowed = 0
        compose.setContent {
            ContextSettingsContent(
                ContextSettingsState(location = false),
                PermissionSteps(background = true, notifications = true),
                {},
                { allowed++ },
                {},
                {},
                {},
                none,
            )
        }

        compose.onNodeWithText("Choose Allow all the time", substring = true).assertIsDisplayed()
        compose.onNodeWithText("Allow all the time").assertIsNotEnabled()
        compose.onNodeWithText("Allow location").performClick()

        assertEquals(1, allowed)
    }

    @Test
    fun grantedPermissionsHaveNoButtonAndAProblemIsShown() {
        compose.setContent {
            ContextSettingsContent(
                ContextSettingsState(
                    location = true,
                    backgroundLocation = true,
                    notifications = true,
                    placeProblem = "Location is turned off on this phone.",
                ),
                PermissionSteps(background = true, notifications = true),
                {},
                {},
                {},
                {},
                {},
                none,
            )
        }

        compose.onNodeWithText("Location: allowed").assertIsDisplayed()
        compose.onNodeWithText("Allow location").assertDoesNotExist()
        compose.onNodeWithText("Location is turned off on this phone.").assertIsDisplayed()
    }

    @Test
    fun olderAndroidVersionsSkipTheStepsTheyDoNotHave() {
        compose.setContent {
            ContextSettingsContent(ContextSettingsState(), PermissionSteps(background = false, notifications = false), {
            }, {}, {}, {}, {}, none)
        }

        compose.onNodeWithText("Location all the time", substring = true).assertDoesNotExist()
        compose.onNodeWithText("Allow notifications").assertDoesNotExist()
    }

    @Test
    fun theSwitchAndTheSystemSettingsButtonReport() {
        var enabled: Boolean? = null
        var opened = 0
        compose.setContent {
            ContextSettingsContent(
                ContextSettingsState(enabled = true),
                PermissionSteps(false, false),
                { enabled = it },
                {},
                {},
                {},
                { opened++ },
                none,
            )
        }

        compose.onNodeWithText("Context rules are on").assertIsDisplayed()
        compose.onNode(isToggleable()).performClick()
        compose.onNodeWithText("Open system settings").performScrollTo().performClick()

        assertEquals(1, opened)
        assertEquals(false, enabled)
    }
}
