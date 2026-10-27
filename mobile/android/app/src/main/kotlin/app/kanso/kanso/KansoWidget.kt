package app.kanso.kanso

import android.content.Context
import android.content.Intent
import android.content.res.Configuration
import android.net.Uri
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.glance.GlanceId
import androidx.glance.GlanceModifier
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.action.clickable
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver
import androidx.glance.appwidget.SizeMode
import androidx.glance.appwidget.action.actionStartActivity
import androidx.glance.appwidget.cornerRadius
import androidx.glance.appwidget.provideContent
import androidx.glance.background
import androidx.glance.layout.Alignment
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.padding
import androidx.glance.layout.size
import androidx.glance.layout.width
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import androidx.glance.unit.ColorProvider
import java.io.File
import org.json.JSONObject

/** The widget's colors, as in `widget.json` (`mobile/docs/capture.md`). */
class WidgetColors(
    val background: Color,
    val surface: Color,
    val text: Color,
    val muted: Color,
    val accent: Color,
) {
    companion object {
        /** Kanso Light, for a widget that has no `widget.json` yet. */
        val LIGHT = WidgetColors(
            background = Color(0xFFF4F1EA),
            surface = Color(0xFFF8F5EF),
            text = Color(0xFF26241F),
            muted = Color(0xFF655F55),
            accent = Color(0xFF5E7895),
        )

        /** Kanso Dark, for a widget that has no `widget.json` yet. */
        val DARK = WidgetColors(
            background = Color(0xFF181816),
            surface = Color(0xFF232320),
            text = Color(0xFFF1EFE7),
            muted = Color(0xFFB7B3A8),
            accent = Color(0xFF7E96B3),
        )

        /**
         * The colors for the light or dark appearance: the app's
         * `widget.json` in [library] when it has one, with Kanso Light or
         * Kanso Dark filling in whatever it lacks or gets wrong.
         */
        fun load(library: File, dark: Boolean): WidgetColors {
            val fallback = if (dark) DARK else LIGHT
            return try {
                val section = JSONObject(File(library, "widget.json").readText(Charsets.UTF_8))
                    .getJSONObject(if (dark) "dark" else "light")
                fun color(key: String, default: Color): Color =
                    parse(section.optString(key)) ?: default
                WidgetColors(
                    background = color("background", fallback.background),
                    surface = color("surface", fallback.surface),
                    text = color("text", fallback.text),
                    muted = color("muted", fallback.muted),
                    accent = color("accent", fallback.accent),
                )
            } catch (_: Exception) {
                fallback
            }
        }

        /** `#RRGGBB` as a color; `null` for anything else. */
        fun parse(value: String): Color? {
            if (value.length != 7 || value[0] != '#') {
                return null
            }
            val rgb = value.substring(1).toIntOrNull(16) ?: return null
            return Color(0xFF000000.toInt() or rgb)
        }
    }
}

/**
 * The home-screen widget: buttons that open Kanso straight into writing.
 *
 * The smallest size has one button, New note. Wider sizes add Voice note and
 * Today. Each button opens `MainActivity` with one of Kanso's links
 * (`kanso://open/new`, `kanso://open/new?voice=1`, `kanso://open/daily`).
 * The colors come from `<filesDir>/kanso/widget.json`, which the app writes
 * when the theme changes; the system's night mode picks light or dark.
 */
class KansoWidget : GlanceAppWidget() {
    override val sizeMode = SizeMode.Responsive(setOf(SMALL, WIDE))

    override suspend fun provideGlance(context: Context, id: GlanceId) {
        val dark = (context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) ==
            Configuration.UI_MODE_NIGHT_YES
        val colors = WidgetColors.load(File(context.filesDir, "kanso"), dark)
        provideContent { Content(context, colors) }
    }

    @Composable
    private fun Content(context: Context, colors: WidgetColors) {
        val wide = androidx.glance.LocalSize.current.width >= WIDE.width
        Row(
            modifier = GlanceModifier
                .fillMaxSize()
                .background(ColorProvider(colors.background))
                .cornerRadius(24.dp)
                .padding(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Action(context, colors, R.drawable.ic_square_pen, R.string.widget_new, NEW, GlanceModifier.defaultWeight())
            if (wide) {
                Spacer(GlanceModifier.width(8.dp))
                Action(context, colors, R.drawable.ic_mic, R.string.widget_voice, VOICE, GlanceModifier.defaultWeight())
                Spacer(GlanceModifier.width(8.dp))
                Action(context, colors, R.drawable.ic_calendar, R.string.widget_today, DAILY, GlanceModifier.defaultWeight())
            }
        }
    }

    @Composable
    private fun Action(
        context: Context,
        colors: WidgetColors,
        icon: Int,
        label: Int,
        link: String,
        modifier: GlanceModifier,
    ) {
        val open = Intent(Intent.ACTION_VIEW, Uri.parse(link))
            .setClassName(context, MainActivity::class.java.name)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        Column(
            modifier = modifier
                .fillMaxSize()
                .background(ColorProvider(colors.surface))
                .cornerRadius(16.dp)
                .clickable(actionStartActivity(open)),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Image(
                provider = ImageProvider(icon),
                contentDescription = null,
                modifier = GlanceModifier.size(24.dp),
                colorFilter = androidx.glance.ColorFilter.tint(ColorProvider(colors.accent)),
            )
            Text(
                text = context.getString(label),
                style = TextStyle(
                    color = ColorProvider(colors.text),
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Medium,
                ),
            )
        }
    }

    companion object {
        private val SMALL = DpSize(110.dp, 40.dp)
        private val WIDE = DpSize(250.dp, 40.dp)
        private const val NEW = "kanso://open/new"
        private const val VOICE = "kanso://open/new?voice=1"
        private const val DAILY = "kanso://open/daily"
    }
}

/** Receives the system's widget updates and hands them to [KansoWidget]. */
class KansoWidgetReceiver : GlanceAppWidgetReceiver() {
    override val glanceAppWidget = KansoWidget()
}
