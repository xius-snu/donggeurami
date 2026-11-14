package town.donggeurami;

import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.graphics.Matrix;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.view.Display;
import android.view.MotionEvent;
import android.view.Surface;
import android.view.SurfaceHolder;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;

import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

import com.google.androidgamesdk.GameActivity;

import java.nio.charset.StandardCharsets;

public class MainActivity extends GameActivity {
    /** How many frames a second the game aims for. */
    private static final float TARGET_FPS = 120f;

    /**
     * The game draws at this fraction of the screen's width and height, and
     * the phone's display hardware scales the picture up to fill the screen,
     * which costs the GPU nothing. 0.75 makes the S26's 1440x3120 1080x2340,
     * what Samsung sets its phones to out of the box (FHD+): 56% of the pixels
     * to draw every frame. Keep in step with {@code RENDER_SCALE} in
     * {@code src/lib.rs}, which keeps the buttons and text their size.
     */
    private static final float RENDER_SCALE = 0.75f;

    /** From the screen's pixels to the game's: what every touch is scaled by. */
    private final Matrix toGame = new Matrix();
    private int gameWidth;
    private int gameHeight;

    static {
        System.loadLibrary("donggeurami_town");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        applyImmersive();
        super.onCreate(savedInstanceState);
        applyImmersive();
        preferTargetRefresh();
        mSurfaceView.addOnLayoutChangeListener(
                (view, left, top, right, bottom, oldLeft, oldTop, oldRight, oldBottom) ->
                        drawSmaller(right - left, bottom - top));
    }

    /**
     * Has the game draw at {@link #RENDER_SCALE} of a view {@code width} by
     * {@code height}. The view itself still fills the screen, and the picture
     * is stretched over it.
     */
    private void drawSmaller(int width, int height) {
        if (width <= 0 || height <= 0) {
            return;
        }
        int w = Math.round(width * RENDER_SCALE);
        int h = Math.round(height * RENDER_SCALE);
        toGame.setScale((float) w / width, (float) h / height);
        if (w != gameWidth || h != gameHeight) {
            gameWidth = w;
            gameHeight = h;
            // Not during the layout this was called from: it asks for another.
            mSurfaceView.post(() -> mSurfaceView.getHolder().setFixedSize(w, h));
        }
    }

    /**
     * A touch comes in the screen's pixels, and the game reads it in its own,
     * the size it draws at, so it is scaled down to land where it was made.
     */
    @Override
    protected boolean processMotionEvent(MotionEvent event) {
        MotionEvent scaled = MotionEvent.obtain(event);
        scaled.transform(toGame);
        boolean handled = super.processMotionEvent(scaled);
        scaled.recycle();
        return handled;
    }

    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        super.onWindowFocusChanged(hasFocus);
        if (hasFocus) {
            applyImmersive();
        }
    }

    /**
     * How far up from the bottom of the screen the soft keyboard reaches, in
     * the screen's pixels, every time the window's insets change: handed to
     * the game ({@code signup::read_keyboard}), which slides the name's box
     * up out from under it. Nothing else about the insets changes: the game
     * is drawn edge to edge regardless.
     */
    private static native void imeInset(int bottom);

    @Override
    public WindowInsetsCompat onApplyWindowInsets(View view, WindowInsetsCompat insets) {
        imeInset(insets.getInsets(WindowInsetsCompat.Type.ime()).bottom);
        return super.onApplyWindowInsets(view, insets);
    }

    /**
     * The soft keyboard's own Enter: its action key ("Done", a tick, an
     * arrow), which is no key press and types no letter. The keyboard asks
     * for what the box is for, {@code GameActivity} hands that here, and
     * the game is told ({@code signup::ime_action}), so that Enter looks
     * for the name typed, or sends the line, as on a desktop's keyboard. A
     * keyboard that is plugged in sends its Enter the same way. Nothing
     * that goes wrong here may stop the game.
     */
    private static native void imeAction(int action);

    @Override
    public void onEditorAction(int action) {
        try {
            imeAction(action);
        } catch (Throwable ignored) {
            // Enter does nothing, as it did before.
        }
        super.onEditorAction(action);
    }

    /**
     * What the game wants copied to the clipboard: your name and tag, from
     * the friends' panel ({@code friends::copy}). The game cannot reach the
     * phone's clipboard itself, so it keeps what is to be copied until this
     * has taken it: how many bytes wait, each of them, and that they have
     * been taken. Numbers only, each way, as with {@link #imeInset}.
     */
    private static native int clipLength();

    private static native int clipByte(int index);

    private static native void clipTaken();

    /** How often the game is asked whether it has anything to copy. */
    private static final long CLIP_POLL_MS = 300;

    private final Handler clipHandler = new Handler(Looper.getMainLooper());
    private final Runnable clipPoll = new Runnable() {
        @Override
        public void run() {
            takeClip();
            clipHandler.postDelayed(this, CLIP_POLL_MS);
        }
    };

    /**
     * Copies whatever the game has waiting to the phone's clipboard. Nothing
     * that goes wrong here may stop the game: with the game's library not
     * there to ask, or the clipboard refusing, nothing is copied and that is
     * all.
     */
    private void takeClip() {
        try {
            int length = clipLength();
            if (length <= 0 || length > 4096) {
                return;
            }
            byte[] bytes = new byte[length];
            for (int i = 0; i < length; i++) {
                bytes[i] = (byte) clipByte(i);
            }
            clipTaken();
            String text = new String(bytes, StandardCharsets.UTF_8);
            ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
            if (clipboard != null) {
                clipboard.setPrimaryClip(ClipData.newPlainText("Donggeurami Town", text));
            }
        } catch (Throwable ignored) {
            // Nothing copied.
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        applyImmersive();
        preferTargetRefresh();
        clipHandler.removeCallbacks(clipPoll);
        clipHandler.postDelayed(clipPoll, CLIP_POLL_MS);
    }

    @Override
    protected void onPause() {
        clipHandler.removeCallbacks(clipPoll);
        super.onPause();
    }

    /**
     * Tells the system the game's frames come {@link #TARGET_FPS} times a
     * second, every time its surface is set up, so that the rate is the
     * game's to choose rather than left to the system's guess.
     */
    @Override
    public void surfaceChanged(SurfaceHolder holder, int format, int width, int height) {
        super.surfaceChanged(holder, format, width, height);
        holder.getSurface().setFrameRate(TARGET_FPS, Surface.FRAME_RATE_COMPATIBILITY_DEFAULT);
    }

    /**
     * Asks for the screen mode that refreshes at {@link #TARGET_FPS} while the
     * game is up. The game draws a frame every time the screen refreshes, so
     * this is what sets its frame rate, and the screen paces the frames: every
     * one is on screen for as long as every other.
     */
    private void preferTargetRefresh() {
        Display display = getDisplay();
        if (display == null) {
            return;
        }
        Display.Mode current = display.getMode();
        Display.Mode chosen = null;
        for (Display.Mode mode : display.getSupportedModes()) {
            boolean sameSize = mode.getPhysicalWidth() == current.getPhysicalWidth()
                    && mode.getPhysicalHeight() == current.getPhysicalHeight();
            if (sameSize && (chosen == null || closer(mode, chosen))) {
                chosen = mode;
            }
        }
        if (chosen == null) {
            return;
        }
        Window window = getWindow();
        WindowManager.LayoutParams params = window.getAttributes();
        if (params.preferredDisplayModeId != chosen.getModeId()) {
            params.preferredDisplayModeId = chosen.getModeId();
            window.setAttributes(params);
        }
    }

    /**
     * Whether {@code a} suits {@link #TARGET_FPS} better than {@code b}: the
     * slowest mode that reaches it, or failing any, the fastest there is.
     */
    private static boolean closer(Display.Mode a, Display.Mode b) {
        boolean aReaches = a.getRefreshRate() >= TARGET_FPS - 1f;
        boolean bReaches = b.getRefreshRate() >= TARGET_FPS - 1f;
        if (aReaches != bReaches) {
            return aReaches;
        }
        return aReaches
                ? a.getRefreshRate() < b.getRefreshRate()
                : a.getRefreshRate() > b.getRefreshRate();
    }

    private void applyImmersive() {
        Window window = getWindow();
        WindowCompat.setDecorFitsSystemWindows(window, false);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            WindowManager.LayoutParams params = window.getAttributes();
            params.layoutInDisplayCutoutMode =
                    WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
            window.setAttributes(params);
        }

        WindowInsetsControllerCompat controller =
                WindowCompat.getInsetsController(window, window.getDecorView());
        controller.hide(WindowInsetsCompat.Type.systemBars());
        controller.setSystemBarsBehavior(
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        );
    }
}
