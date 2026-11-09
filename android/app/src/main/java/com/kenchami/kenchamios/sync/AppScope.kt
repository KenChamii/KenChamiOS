package com.kenchami.kenchamios.sync

import javax.inject.Qualifier

/** The coroutine scope that lives as long as the app process. */
@Qualifier
@Retention(AnnotationRetention.RUNTIME)
annotation class AppScope
