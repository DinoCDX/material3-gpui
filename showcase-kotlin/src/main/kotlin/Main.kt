import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Badge
import androidx.compose.material3.Button
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.FilledTonalIconButton
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.LargeFloatingActionButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedIconButton
import androidx.compose.material3.SmallFloatingActionButton
import androidx.compose.material3.AssistChip
import androidx.compose.material3.ElevatedAssistChip
import androidx.compose.material3.ElevatedSuggestionChip
import androidx.compose.material3.FilterChip
import androidx.compose.material3.InputChip
import androidx.compose.material3.RangeSlider
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Slider
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Switch
import androidx.compose.material3.TriStateCheckbox
import androidx.compose.ui.state.ToggleableState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.Typography
import androidx.compose.material3.ColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import com.materialkolor.dynamicColorScheme
import java.lang.management.ManagementFactory
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import java.io.File
import kotlin.concurrent.thread

const val FRAMES_PER_REPORT = 60
const val DEFAULT_SPEC = "../spec/showcase.json"

fun report(event: JsonObject) {
    println(event.toString())
    System.out.flush()
}

fun field(item: JsonObject, key: String): String = item[key]?.jsonPrimitive?.content ?: ""

fun seedColor(hex: String): Color {
    val value = hex.removePrefix("#").toLong(16)
    return Color(0xFF000000L or value)
}

fun textStyleFor(name: String, typography: Typography): TextStyle {
    val emphasized = name.endsWith("Emphasized")
    val base = name.removeSuffix("Emphasized")
    val style = when (base) {
        "DisplayLarge" -> typography.displayLarge
        "DisplayMedium" -> typography.displayMedium
        "DisplaySmall" -> typography.displaySmall
        "HeadlineLarge" -> typography.headlineLarge
        "HeadlineMedium" -> typography.headlineMedium
        "HeadlineSmall" -> typography.headlineSmall
        "TitleLarge" -> typography.titleLarge
        "TitleMedium" -> typography.titleMedium
        "TitleSmall" -> typography.titleSmall
        "BodyLarge" -> typography.bodyLarge
        "BodyMedium" -> typography.bodyMedium
        "BodySmall" -> typography.bodySmall
        "LabelLarge" -> typography.labelLarge
        "LabelMedium" -> typography.labelMedium
        "LabelSmall" -> typography.labelSmall
        else -> typography.bodyLarge
    }
    return if (emphasized) style.copy(fontWeight = FontWeight.Bold) else style
}

fun swatchColor(name: String, scheme: ColorScheme): Color = when (name) {
    "primary" -> scheme.primary
    "onPrimary" -> scheme.onPrimary
    "primaryContainer" -> scheme.primaryContainer
    "secondary" -> scheme.secondary
    "tertiary" -> scheme.tertiary
    "error" -> scheme.error
    "surface" -> scheme.surface
    "surfaceVariant" -> scheme.surfaceVariant
    "outline" -> scheme.outline
    else -> scheme.onSurface
}

fun buttonLabelStyle(size: String, typography: Typography): TextStyle = when (size) {
    "medium" -> typography.titleMedium
    "large" -> typography.headlineSmall
    "xlarge" -> typography.headlineLarge
    else -> typography.labelLarge
}

fun buttonPadding(size: String): PaddingValues = when (size) {
    "medium" -> PaddingValues(start = 24.dp, end = 24.dp)
    "large" -> PaddingValues(start = 48.dp, end = 48.dp)
    "xlarge" -> PaddingValues(start = 64.dp, end = 64.dp)
    else -> PaddingValues(start = 16.dp, end = 16.dp)
}

fun iconGap(size: String) = when (size) {
    "large" -> 12.dp
    "xlarge" -> 16.dp
    else -> 8.dp
}

fun buttonHeight(size: String) = when (size) {
    "xsmall" -> 32.dp
    "medium" -> 56.dp
    "large" -> 96.dp
    "xlarge" -> 136.dp
    else -> 40.dp
}

@Composable
fun ShowcaseItem(item: JsonObject, scheme: ColorScheme, typography: Typography) {
    when (field(item, "kind")) {
        "text" -> Text(
            text = field(item, "text"),
            style = textStyleFor(field(item, "style"), typography),
        )
        "swatch" -> Surface(
            color = swatchColor(field(item, "role"), scheme),
            modifier = Modifier.size(width = 140.dp, height = 44.dp),
        ) {
            Text(
                text = field(item, "name"),
                fontSize = 11.sp,
                color = scheme.onSurface,
                textAlign = TextAlign.Center,
            )
        }
        "button" -> {
            val size = field(item, "size")
            val modifier = Modifier.height(buttonHeight(size))
            val enabled = item["enabled"]?.jsonPrimitive?.content != "false"
            val gap = iconGap(size)
            val labelStyle = buttonLabelStyle(size, typography)
            val padding = buttonPadding(size)
            val leading: @Composable () -> Unit = {
                if (item["leading"]?.jsonPrimitive?.content == "true") {
                    Text("+")
                    Spacer(Modifier.width(gap))
                }
            }
            val label: @Composable () -> Unit = {
                ProvideTextStyle(labelStyle) { Text(field(item, "label")) }
            }
            when (field(item, "variant")) {
                "tonal" -> FilledTonalButton(onClick = {}, modifier = modifier, enabled = enabled, contentPadding = padding) {
                    leading(); label()
                }
                "elevated" -> ElevatedButton(onClick = {}, modifier = modifier, enabled = enabled, contentPadding = padding) {
                    leading(); label()
                }
                "outlined" -> OutlinedButton(onClick = {}, modifier = modifier, enabled = enabled, contentPadding = padding) {
                    leading(); label()
                }
                "text" -> TextButton(onClick = {}, modifier = modifier, enabled = enabled, contentPadding = padding) {
                    leading(); label()
                }
                else -> Button(onClick = {}, modifier = modifier, enabled = enabled, contentPadding = padding) {
                    leading(); label()
                }
            }
        }
        "icon_button" -> when (field(item, "variant")) {
            "filled" -> FilledIconButton(onClick = {}) { Text("+") }
            "tonal" -> FilledTonalIconButton(onClick = {}) { Text("+") }
            "outlined" -> OutlinedIconButton(onClick = {}) { Text("+") }
            else -> IconButton(onClick = {}) { Text("+") }
        }
        "fab" -> {
            val extended = item["extended"]?.jsonPrimitive?.content
            val secondary = field(item, "color") == "secondary"
            val container = if (secondary) scheme.secondaryContainer else scheme.primaryContainer
            val content = if (secondary) scheme.onSecondaryContainer else scheme.onPrimaryContainer
            if (extended != null) {
                ExtendedFloatingActionButton(
                    text = { Text(extended) },
                    icon = { Text("+") },
                    onClick = {},
                    containerColor = container,
                    contentColor = content,
                )
            } else {
                when (field(item, "size")) {
                    "small" -> SmallFloatingActionButton(onClick = {}, containerColor = container, contentColor = content) { Text("+") }
                    "large" -> LargeFloatingActionButton(onClick = {}, containerColor = container, contentColor = content) { Text("+") }
                    else -> FloatingActionButton(onClick = {}, containerColor = container, contentColor = content) { Text("+") }
                }
            }
        }
        "checkbox" -> {
            val enabled = item["enabled"]?.jsonPrimitive?.content != "false"
            var state by remember { mutableStateOf(field(item, "state")) }
            TriStateCheckbox(
                state = when (state) {
                    "checked" -> ToggleableState.On
                    "indeterminate" -> ToggleableState.Indeterminate
                    else -> ToggleableState.Off
                },
                onClick = { state = if (state == "checked") "unchecked" else "checked" },
                enabled = enabled,
            )
        }
        "radio" -> {
            val enabled = item["enabled"]?.jsonPrimitive?.content != "false"
            var selected by remember { mutableStateOf(item["selected"]?.jsonPrimitive?.content == "true") }
            RadioButton(selected = selected, onClick = { selected = true }, enabled = enabled)
        }
        "switch" -> {
            val enabled = item["enabled"]?.jsonPrimitive?.content != "false"
            val showIcon = item["icon"]?.jsonPrimitive?.content == "true"
            var on by remember { mutableStateOf(item["selected"]?.jsonPrimitive?.content == "true") }
            Switch(
                checked = on,
                onCheckedChange = { on = it },
                enabled = enabled,
                thumbContent = if (showIcon) ({ Text(if (on) "✓" else "×") }) else null,
            )
        }
        "slider" -> {
            val min = item["min"]?.jsonPrimitive?.content?.toFloat() ?: 0f
            val max = item["max"]?.jsonPrimitive?.content?.toFloat() ?: 100f
            val steps = item["steps"]?.jsonPrimitive?.content?.toInt() ?: 0
            val width = item["width"]?.jsonPrimitive?.content?.toFloat() ?: 240f
            val enabled = item["enabled"]?.jsonPrimitive?.content != "false"
            var value by remember {
                mutableStateOf(item["value"]?.jsonPrimitive?.content?.toFloat() ?: min)
            }
            Slider(
                value = value,
                onValueChange = { value = it },
                valueRange = min..max,
                steps = steps,
                enabled = enabled,
                modifier = Modifier.width(width.dp),
            )
        }
        "range_slider" -> {
            val min = item["min"]?.jsonPrimitive?.content?.toFloat() ?: 0f
            val max = item["max"]?.jsonPrimitive?.content?.toFloat() ?: 100f
            val steps = item["steps"]?.jsonPrimitive?.content?.toInt() ?: 0
            val width = item["width"]?.jsonPrimitive?.content?.toFloat() ?: 240f
            val enabled = item["enabled"]?.jsonPrimitive?.content != "false"
            var range by remember {
                mutableStateOf(
                    (item["low"]?.jsonPrimitive?.content?.toFloat() ?: min)..
                        (item["high"]?.jsonPrimitive?.content?.toFloat() ?: max)
                )
            }
            RangeSlider(
                value = range,
                onValueChange = { range = it },
                valueRange = min..max,
                steps = steps,
                enabled = enabled,
                modifier = Modifier.width(width.dp),
            )
        }
        "chip" -> {
            val label = field(item, "label")
            val leading = item["leading"]?.jsonPrimitive?.content
            val trailing = item["trailing"]?.jsonPrimitive?.content
            val elevated = item["elevated"]?.jsonPrimitive?.content == "true"
            val selected0 = item["selected"]?.jsonPrimitive?.content == "true"
            var selected by remember { mutableStateOf(selected0) }
            val leadingIcon: (@Composable () -> Unit)? = leading?.let { glyph -> { Text(glyph) } }
            when (field(item, "variant")) {
                "filter" -> FilterChip(
                    selected = selected,
                    onClick = { selected = !selected },
                    label = { Text(label) },
                    leadingIcon = leadingIcon,
                )
                "input" -> InputChip(
                    selected = selected,
                    onClick = { selected = !selected },
                    label = { Text(label) },
                    trailingIcon = trailing?.let { glyph -> { Text(glyph) } },
                )
                "suggestion" -> if (elevated) {
                    ElevatedSuggestionChip(onClick = {}, label = { Text(label) }, icon = leadingIcon)
                } else {
                    SuggestionChip(onClick = {}, label = { Text(label) }, icon = leadingIcon)
                }
                else -> if (elevated) {
                    ElevatedAssistChip(onClick = {}, label = { Text(label) }, leadingIcon = leadingIcon)
                } else {
                    AssistChip(onClick = {}, label = { Text(label) }, leadingIcon = leadingIcon)
                }
            }
        }
        "divider" -> HorizontalDivider(modifier = Modifier.width(160.dp))
        "badge" -> when (val content = field(item, "content")) {
            "dot" -> Badge()
            else -> Badge { Text(content) }
        }
        else -> Spacer(Modifier.size(0.dp))
    }
}

@Composable
fun FrameReporter(startupMs: Double) {
    LaunchedEffect(Unit) {
        var last = -1L
        val frameTimes = mutableListOf<Double>()
        var reportedStartup = false
        while (true) {
            withFrameNanos { now ->
                if (!reportedStartup) {
                    reportedStartup = true
                    report(buildJsonObject { put("event", "ready"); put("startup_ms", startupMs) })
                }
                if (last >= 0) {
                    frameTimes.add((now - last) / 1_000_000.0)
                    if (frameTimes.size >= FRAMES_PER_REPORT) {
                        val sorted = frameTimes.sorted()
                        val count = sorted.size
                        val p95 = sorted[((count - 1) * 0.95).toInt()]
                        report(buildJsonObject {
                            put("event", "frames")
                            put("count", count)
                            put("avg_ms", sorted.average())
                            put("min_ms", sorted.first())
                            put("p95_ms", p95)
                            put("max_ms", sorted.last())
                        })
                        frameTimes.clear()
                    }
                }
                last = now
            }
        }
    }
}

@OptIn(androidx.compose.foundation.layout.ExperimentalLayoutApi::class, ExperimentalMaterial3Api::class)
@Composable
fun ShowcaseApp(spec: JsonObject, toggles: StateFlow<Int>, startupMs: Double) {
    val toggleCount by toggles.collectAsState()
    var dark by remember { mutableStateOf(false) }
    LaunchedEffect(toggleCount) {
        if (toggleCount > 0) dark = !dark
    }
    val seed = seedColor(spec["seed"]?.jsonPrimitive?.content ?: "#4285F4")
    val scheme = dynamicColorScheme(seedColor = seed, isDark = dark)
    FrameReporter(startupMs)

    MaterialTheme(colorScheme = scheme) {
        Surface(color = scheme.background, modifier = Modifier.fillMaxSize()) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .verticalScroll(rememberScrollState())
                    .padding(24.dp),
                verticalArrangement = Arrangement.spacedBy(28.dp),
            ) {
                spec["screens"]!!.jsonArray.forEach { screenElement ->
                    val screen = screenElement.jsonObject
                    Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        Text(
                            text = field(screen, "title"),
                            fontSize = 16.sp,
                            color = scheme.onBackground,
                        )
                        FlowRow(
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                            verticalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            screen["items"]!!.jsonArray.forEach { itemElement ->
                                ShowcaseItem(itemElement.jsonObject, scheme, MaterialTheme.typography)
                            }
                        }
                    }
                }
            }
        }
    }
}

fun main() = application {
    val specPath = System.getenv("SHOWCASE_SPEC") ?: DEFAULT_SPEC
    val spec = Json.parseToJsonElement(File(specPath).readText()).jsonObject
    val startupMs = ManagementFactory.getRuntimeMXBean().uptime.toDouble()
    val toggles = MutableStateFlow(0)
    thread(isDaemon = true) {
        System.`in`.bufferedReader().lineSequence().forEach { line ->
            if (line.contains("toggle_theme")) toggles.value = toggles.value + 1
        }
    }
    Window(
        onCloseRequest = ::exitApplication,
        title = field(spec, "name"),
        state = rememberWindowState(size = DpSize(1280.dp, 2000.dp)),
    ) {
        ShowcaseApp(spec, toggles, startupMs)
    }
}
