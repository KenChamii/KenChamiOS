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
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.kenchami.kenchamios.ui.common.MOTION_MS
import com.kenchami.kenchamios.ui.media.EntryDetailScreen
import com.kenchami.kenchamios.ui.media.MediaScreen
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

private const val SLIDE_DIVISOR = 12

@Composable
fun AppScaffold() {
    val navController = rememberNavController()
    val backStack by navController.currentBackStackEntryAsState()
    val current = backStack?.destination?.route
    var quickAdd by rememberSaveable { mutableStateOf<QuickAddTab?>(null) }
    val openQuickAdd = { tab: QuickAddTab -> quickAdd = tab }

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
                        openProject = { navController.navigate(Routes.project(it)) },
                        openEntry = { navController.navigate(Routes.entry(it)) },
                        quickAdd = openQuickAdd,
                        openSync = { navController.navigate(Routes.SYNC) },
                    ),
                    onOpenRules = { navController.navigate(Routes.RULES) },
                )
            }
            composable(Destination.Projects.route) {
                ProjectsScreen(
                    onOpenProject = { navController.navigate(Routes.project(it)) },
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
        }
    }

    quickAdd?.let { QuickAddSheet(it, onDismiss = { quickAdd = null }) }
}
