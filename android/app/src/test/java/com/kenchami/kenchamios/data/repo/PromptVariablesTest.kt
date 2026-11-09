package com.kenchami.kenchamios.data.repo

import com.kenchami.kenchamios.projectSummary
import org.junit.Assert.assertEquals
import org.junit.Test

class PromptVariablesTest {
    private val project = projectSummary(name = "Demo").project.copy(
        description = "A small app",
        repoPath = "C:/code/demo",
        tags = listOf("kotlin", "android"),
    )

    @Test
    fun fillsEveryProjectVariableWithOrWithoutSpaces() {
        val body = "{{ project.name }}|{{project.description}}|{{  project.stage }}|{{ project.repo_path }}|{{ project.tags }}"

        assertEquals("Demo|A small app|build|C:/code/demo|kotlin, android", fillProjectVariables(body, project))
    }

    @Test
    fun leavesOtherVariablesAsTheyAre() {
        assertEquals("{{ task.title }} in Demo", fillProjectVariables("{{ task.title }} in {{ project.name }}", project))
    }

    @Test
    fun withoutAProjectTheTextIsUnchanged() {
        assertEquals("{{ project.name }}", fillProjectVariables("{{ project.name }}", null))
    }

    @Test
    fun aMissingRepoPathBecomesEmpty() {
        assertEquals("path:", fillProjectVariables("path:{{ project.repo_path }}", project.copy(repoPath = null)))
    }
}
