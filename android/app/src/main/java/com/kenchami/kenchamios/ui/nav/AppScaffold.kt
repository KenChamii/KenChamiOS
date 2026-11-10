package com.kenchami.kenchamios.ui.nav

import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarResult
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.kenchami.kenchamios.data.model.Inbox
import com.kenchami.kenchamios.ui.common.MOTION_MS
import com.kenchami.kenchamios.ui.common.UndoBus
import com.kenchami.kenchamios.ui.habits.HabitsScreen
import com.kenchami.kenchamios.ui.inbox.InboxScreen
import com.kenchami.kenchamios.ui.media.EntryDetailScreen
import com.kenchami.kenchamios.ui.media.MediaScreen
import com.kenchami.kenchamios.ui.notes.DailyNotesScreen
import com.kenchami.kenchamios.ui.planner.PlanDayScreen
import com.kenchami.kenchamios.ui.planner.ReviewDaySheet
import com.kenchami.kenchamios.ui.projects.ProjectDetailScreen
import com.kenchami.kenchamios.ui.projects.ProjectsScreen
import com.kenchami.kenchamios.ui.prompts.PromptsScreen
import com.kenchami.kenchamios.ui.quickadd.QuickAddSheet
import com.kenchami.kenchamios.ui.quickadd.QuickAddTab
import com.kenchami.kenchamios.ui.rules.ContextSettingsScreen
import com.kenchami.kenchamios.ui.rules.RuleEditorScreen
import com.kenchami.kenchamios.ui.rules.RulesScreen
import com.kenchami.kenchamios.ui.sync.SyncScreen
import com.kenchami.kenchamios.ui.theme.tint
import com.kenchami.kenchamios.ui.today.TodayNav
import com.kenchami.kenchamios.ui.today.TodayScreen
import com.kenchami.kenchamios.ui.trash.TrashScreen
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch

private const val SLIDE_DIVISOR = 12

@Composable
fun AppScaffold(undo: UndoBus, sharedText: String?, onSharedTextUsed: () -> Unit) {
    val navController = rememberNavController()
    val backStack by navController.currentBackStackEntryAsState()
    val current = backStack?.destination?.route
    var quickAdd by rememberSaveable { mutableStateOf<QuickAddTab?>(null) }
    var quickAddText by rememberSaveable { mutableStateOf("") }
    var review by rememberSaveable { mutableStateOf(false) }
    val openQuickAdd = { tab: QuickAddTab -> quickAdd = tab }
    val openProject = { id: String -> navController.navigate(if (id == Inbox.ID) Routes.INBOX else Routes.project(id)) }
    val snackbar = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()

    LaunchedEffect(sharedText) {
        if (sharedText != null) {
            quickAddText = sharedText
            quickAdd = QuickAddTab.Task
            onSharedTextUsed()
        }
    }
    LaunchedEffect(undo) {
        undo.offers.collectLatest { offer ->
            val result = snackbar.showSnackbar(offer.message, actionLabel = "Undo", duration = SnackbarDuration.Short)
            if (result == SnackbarResult.ActionPerformed) {
                scope.launch {
                    runCatching { offer.undo() }.onFailure { snackbar.showSnackbar("Could not undo: ${it.message}") }
                }
            }
        }
    }

    Scaffold(
        bottomBar = {
            NavigationBar(containerColor = MaterialTheme.colorScheme.background, tonalElevation = 0.dp) {
                Destination.entries.forEach { destination ->
                    val tint = destination.area?.tint
                    NavigationBarItem(
                        selected = current == destination.route,
                        onClick = {
                            navController.navigate(destination.route) {
                                popUpTo(navController.graph.findStartDestination().id) { saveState = true }
                                launchSingleTop = true
                                restoreState = true
                            }
                        },
                        icon = { Icon(destination.icon, contentDescription = null) },
                        label = { Text(destination.label) },
                        colors = NavigationBarItemDefaults.colors(
                            indicatorColor = tint?.container ?: MaterialTheme.colorScheme.surfaceContainerHigh,
                            selectedIconColor = tint?.content ?: MaterialTheme.colorScheme.onSurface,
                            unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
                            unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant,
                        ),
                    )
                }
            }
        },
        snackbarHost = { SnackbarHost(snackbar) },
        containerColor = MaterialTheme.colorScheme.background,
        contentWindowInsets = WindowInsets(0),
    ) { padding ->
        NavHost(
            navController = navController,
            startDestination = Destination.Today.route,
            modifier = Modifier.padding(padding),
            enterTransition = { fadeIn(tween(MOTION_MS)) + slideInHorizontally(tween(MOTION_MS)) { it / SLIDE_DIVISOR } },
            exitTransition = { fadeOut(tween(MOTION_MS)) + slideOutHorizontally(tween(MOTION_MS)) { -it / SLIDE_DIVISOR } },
            popEnterTransition = { fadeIn(tween(MOTION_MS)) + slideInHorizontally(tween(MOTION_MS)) { -it / SLIDE_DIVISOR } },
            popExitTransition = { fadeOut(tween(MOTION_MS)) + slideOutHorizontally(tween(MOTION_MS)) { it / SLIDE_DIVISOR } },
        ) {
            composable(Destination.Today.route) {
                TodayScreen(
                    nav = TodayNav(
                        openProject = openProject,
                        openEntry = { navController.navigate(Routes.entry(it)) },
                        quickAdd = openQuickAdd,
                        openSync = { navController.navigate(Routes.SYNC) },
                        openInbox = { navController.navigate(Routes.INBOX) },
                        openHabits = { navController.navigate(Routes.HABITS) },
                        openTrash = { navController.navigate(Routes.TRASH) },
                        openPlan = { navController.navigate(Routes.PLAN) },
                        openReview = { review = true },
                        openNotes = { navController.navigate(Routes.NOTES) },
                    ),
                    onOpenRules = { navController.navigate(Routes.RULES) },
                )
            }
            composable(Destination.Projects.route) {
                ProjectsScreen(
                    onOpenProject = openProject,
                    onQuickAdd = { openQuickAdd(QuickAddTab.Idea) },
                )
            }
            composable(Destination.Prompts.route) { PromptsScreen(onQuickAdd = { openQuickAdd(QuickAddTab.Prompt) }) }
            composable(Destination.Media.route) {
                MediaScreen(onOpenEntry = { navController.navigate(Routes.entry(it)) }, onQuickAdd = { openQuickAdd(QuickAddTab.Media) })
            }
            composable(Routes.PROJECT) { ProjectDetailScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.ENTRY) { EntryDetailScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.SYNC) { SyncScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.RULES) {
                RulesScreen(
                    onBack = { navController.popBackStack() },
                    onOpenRule = { navController.navigate(Routes.rule(it)) },
                    onOpenSettings = { navController.navigate(Routes.RULES_SETTINGS) },
                )
            }
            composable(Routes.RULE) { RuleEditorScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.RULES_SETTINGS) { ContextSettingsScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.INBOX) { InboxScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.HABITS) { HabitsScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.TRASH) { TrashScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.PLAN) { PlanDayScreen(onBack = { navController.popBackStack() }) }
            composable(Routes.NOTES) { DailyNotesScreen(onBack = { navController.popBackStack() }) }
        }
    }

    quickAdd?.let {
        QuickAddSheet(it, onDismiss = {
            quickAdd = null
            quickAddText = ""
        }, initialText = quickAddText)
    }
    if (review) {
        ReviewDaySheet(onDismiss = { review = false }, onOpenNote = { navController.navigate(Routes.NOTES) })
    }
}
