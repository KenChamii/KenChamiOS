package com.kenchami.kenchamios.ui.quickadd

import com.kenchami.kenchamios.core.startOfDay
import com.kenchami.kenchamios.data.db.ProjectSummary
import com.kenchami.kenchamios.data.model.Priority
import com.kenchami.kenchamios.data.repo.TaskRepository
import java.time.LocalDate
import java.time.ZoneId

/** The project a `#name` points at, by slug or name, ignoring case. */
fun matchProject(name: String, projects: List<ProjectSummary>): ProjectSummary? {
    val wanted = name.lowercase()
    return projects.firstOrNull { it.project.slug == wanted } ?: projects.firstOrNull { it.project.name.lowercase() == wanted }
}

/** What a quick-add line asks for; [project] is the text after `#`, as typed. */
data class QuickTask(val title: String, val due: LocalDate?, val priority: Priority?, val project: String?, val recurrence: String?)

/** A task as the quick add form hands it over; [dueAt] is local midnight. */
data class NewTask(val projectId: String, val title: String, val priority: Priority, val dueAt: Long?, val recurrence: String?)

fun QuickTask.toNewTask(projectId: String, zone: ZoneId) =
    NewTask(projectId, title, priority ?: Priority.Medium, due?.let { startOfDay(it, zone) }, recurrence)

suspend fun TaskRepository.create(task: NewTask): String = create(task.projectId, task.title, task.priority, task.dueAt, task.recurrence)
