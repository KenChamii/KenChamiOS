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
import com.kenchami.kenchamios.ui.media.EntryDetailScreen
import com.kenchami.kenchamios.ui.media.MediaScreen
import com.kenchami.kenchamios.ui.projects.ProjectDetailScreen
import com.kenchami.kenchamios.ui.projects.ProjectsScreen
import com.kenchami.kenchamios.ui.prompts.PromptsScreen
import com.kenchami.kenchamios.ui.quickadd.QuickAddSheet
import com.kenchami.kenchamios.ui.rules.ContextSettingsScreen
import com.kenchami.kenchamios.ui.rules.RuleEditorScreen
import com.kenchami.kenchamios.ui.rules.RulesScreen
import com.kenchami.kenchamios.ui.sync.SyncScreen
import com.kenchami.kenchamios.ui.today.TodayScreen

// compose scales tween durations by the system animator setting, so reduced motion shortens these
private const val SCREEN_MS = 200
private const val SLIDE_DIVISOR = 12

@Composable
fun AppScaffold() {
    val navController = rememberNavController()
    val backStack by navController.currentBackStackEntryAsState()
    val current = backStack?.destination?.route
    var quickAdd by rememberSaveable { mutableStateOf(false) }
    val openQuickAdd = { quickAdd = true }

    Scaffold(
        bottomBar = {
            NavigationBar(containerColor = MaterialTheme.colorScheme.background, tonalElevation = 0.dp) {
                Destination.entries.forEach { destination ->
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
                            indicatorColor = MaterialTheme.colorScheme.surfaceContainerHigh,
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
            enterTransition = { fadeIn(tween(SCREEN_MS)) + slideInHorizontally(tween(SCREEN_MS)) { it / SLIDE_DIVISOR } },
            exitTransition = { fadeOut(tween(SCREEN_MS)) + slideOutHorizontally(tween(SCREEN_MS)) { -it / SLIDE_DIVISOR } },
            popEnterTransition = { fadeIn(tween(SCREEN_MS)) + slideInHorizontally(tween(SCREEN_MS)) { -it / SLIDE_DIVISOR } },
            popExitTransition = { fadeOut(tween(SCREEN_MS)) + slideOutHorizontally(tween(SCREEN_MS)) { it / SLIDE_DIVISOR } },
        ) {
            composable(Destination.Today.route) {
                TodayScreen(
                    onOpenProject = { navController.navigate(Routes.project(it)) },
                    onOpenEntry = { navController.navigate(Routes.entry(it)) },
                    onQuickAdd = openQuickAdd,
                    onOpenSync = { navController.navigate(Routes.SYNC) },
                    onOpenRules = { navController.navigate(Routes.RULES) },
                )
            }
            composable(Destination.Projects.route) {
                ProjectsScreen(onOpenProject = { navController.navigate(Routes.project(it)) }, onQuickAdd = openQuickAdd)
            }
            composable(Destination.Prompts.route) { PromptsScreen(onQuickAdd = openQuickAdd) }
            composable(Destination.Media.route) {
                MediaScreen(onOpenEntry = { navController.navigate(Routes.entry(it)) }, onQuickAdd = openQuickAdd)
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

    if (quickAdd) QuickAddSheet(onDismiss = { quickAdd = false })
}
