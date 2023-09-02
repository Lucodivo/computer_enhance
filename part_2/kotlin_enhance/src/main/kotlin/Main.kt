import kotlin.time.TimeSource
import kotlin.math.min

fun main(args: Array<String>) {

//    val timeSource = TimeSource.Monotonic
//    var shortestElapsed = Long.MAX_VALUE
//    for (i in 0..10_000_000) {
//        val elapsed = timeSource.markNow().elapsedNow().inWholeNanoseconds
//        shortestElapsed = min(shortestElapsed, elapsed)
//    }

    var shortestElapsed = Long.MAX_VALUE
    for (i in 0..10_000_000) {
        val start = System.nanoTime()
        val elapsed = System.nanoTime() - start
        shortestElapsed = min(shortestElapsed, elapsed)
    }

    val elapsedSeconds = shortestElapsed.toDouble() / 1_000_000_000.0

    println("Shortest possible interval: $elapsedSeconds seconds")
    //println("Program arguments: ${args.joinToString()}")
}