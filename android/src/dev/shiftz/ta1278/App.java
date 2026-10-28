package dev.shiftz.ta1278;

import android.app.NativeActivity;
import android.os.Bundle;
import android.os.Build;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.view.HapticFeedbackConstants;
import android.widget.Toast;

public final class App extends NativeActivity {
    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        getWindow().getDecorView().setHapticFeedbackEnabled(true);
        immersive();
    }
    @Override public void onWindowFocusChanged(boolean focused) {
        super.onWindowFocusChanged(focused);
        if (focused) immersive();
    }
    @SuppressWarnings("deprecation")
    private void immersive() {
        if (Build.VERSION.SDK_INT >= 28) {
            WindowManager.LayoutParams attributes = getWindow().getAttributes();
            attributes.layoutInDisplayCutoutMode = Build.VERSION.SDK_INT >= 30
                ? WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS
                : WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
            getWindow().setAttributes(attributes);
        }
        if (Build.VERSION.SDK_INT >= 30) {
            getWindow().setDecorFitsSystemWindows(false);
            WindowInsetsController controller = getWindow().getInsetsController();
            if (controller != null) {
                controller.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
                controller.hide(WindowInsets.Type.systemBars());
            }
        } else {
            getWindow().getDecorView().setSystemUiVisibility(View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                | View.SYSTEM_UI_FLAG_FULLSCREEN | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                | View.SYSTEM_UI_FLAG_LAYOUT_STABLE | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION);
        }
    }
    // JNI entry: native input is on another thread. Respect Android's haptics preference.
    public boolean quiet() {
        return getIntent().getBooleanExtra("quiet", false);
    }
    public void feedback() {
        runOnUiThread(() -> {
            if (hasWindowFocus()) {
                getWindow().getDecorView().performHapticFeedback(HapticFeedbackConstants.VIRTUAL_KEY);
            }
        });
    }
    // Called by a decoder worker, never the UI thread. Bound decode size before allocation.
    public int[] picture(byte[] bytes) {
        BitmapFactory.Options options = new BitmapFactory.Options();
        options.inJustDecodeBounds = true;
        BitmapFactory.decodeByteArray(bytes, 0, bytes.length, options);
        int width = options.outWidth, height = options.outHeight;
        if (width <= 0 || height <= 0 || (long) width * height > 32000000L) return null;
        options.inJustDecodeBounds = false;
        options.inPreferredConfig = Bitmap.Config.ARGB_8888;
        options.inSampleSize = 1;
        while (Math.max(width, height) / options.inSampleSize > 480) options.inSampleSize *= 2;
        Bitmap source = BitmapFactory.decodeByteArray(bytes, 0, bytes.length, options);
        if (source == null) return null;
        float scale = Math.min(1f, 240f / Math.max(source.getWidth(), source.getHeight()));
        Bitmap bitmap = Bitmap.createScaledBitmap(source,
            Math.max(1, Math.round(source.getWidth() * scale)),
            Math.max(1, Math.round(source.getHeight() * scale)), true);
        int w = bitmap.getWidth(), h = bitmap.getHeight();
        int[] pixels = new int[2 + w * h];
        pixels[0] = w;
        pixels[1] = h;
        bitmap.getPixels(pixels, 2, w, 0, 0, w, h);
        bitmap.recycle();
        if (source != bitmap) source.recycle();
        return pixels;
    }
    public void failure(String message) {
        runOnUiThread(() -> Toast.makeText(this, message, Toast.LENGTH_LONG).show());
    }
}
