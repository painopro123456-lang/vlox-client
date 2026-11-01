package dev.vlox.client;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keybinding.v1.KeyBindingHelper;
import net.minecraft.client.option.KeyBinding;
import net.minecraft.client.MinecraftClient;
import net.fabricmc.fabric.api.client.rendering.v1.HudRenderCallback;
import net.minecraft.client.util.InputUtil;
import org.lwjgl.glfw.GLFW;

public final class VloxClient implements ClientModInitializer {
    public static boolean showCoordinates = true;
    public static boolean showFps = true;
    public static boolean cleanChat = false;

    @Override
    public void onInitializeClient() {
        KeyBinding openMenu = KeyBindingHelper.registerKeyBinding(new KeyBinding(
            "key.vlox.open_menu", InputUtil.Type.KEYSYM, GLFW.GLFW_KEY_RIGHT_SHIFT,
            "category.vlox.main"
        ));
        HudRenderCallback.EVENT.register((drawContext, tickCounter) -> {
            MinecraftClient client = MinecraftClient.getInstance();
            if (client.player == null) return;
            int y = 8;
            if (showFps) {
                drawContext.drawText(client.textRenderer, "VLOX  |  " + client.getCurrentFps() + " FPS", 8, y, 0xFF75A8FF, true);
                y += 12;
            }
            if (showCoordinates) {
                drawContext.drawText(client.textRenderer, "XYZ  " + client.player.getBlockPos().toShortString(), 8, y, 0xFFEAF1FC, true);
            }
        });
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            while (openMenu.wasPressed()) {
                if (client.currentScreen == null) client.setScreen(new VloxScreen());
                else if (client.currentScreen instanceof VloxScreen) client.setScreen(null);
            }
        });
    }
}
