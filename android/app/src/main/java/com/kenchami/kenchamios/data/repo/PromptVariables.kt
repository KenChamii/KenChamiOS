package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.data.db.entity.ProjectEntity

private val PROJECT_VARIABLE = Regex("""\{\{\s*project\.(name|description|stage|repo_path|tags)\s*\}\}""")

/**
 * Fills the `{{ project.* }}` variables of a prompt, the ones the desktop fills too.
 * Other variables stay as written, the phone has no form to ask for them.
 */
fun fillProjectVariables(body: String, project: ProjectEntity?): String {
    if (project == null) return body
    return PROJECT_VARIABLE.replace(body) { match ->
        when (match.groupValues[1]) {
            "name" -> project.name
            "description" -> project.description
            "stage" -> project.stage.wire
            "repo_path" -> project.repoPath.orEmpty()
            else -> project.tags.joinToString(", ")
        }
    }
}
