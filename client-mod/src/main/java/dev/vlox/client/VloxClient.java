package dev.vlox.client;

import com.mojang.blaze3d.platform.InputConstants;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.fabricmc.fabric.api.client.rendering.v1.hud.VanillaHudElements;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.resources.Identifier;
import org.lwjgl.glfw.GLFW;

public final class VloxClient implements ClientModInitializer {
    public static boolean showCoordinates = true;
    public static boolean showFps = true;

    private static final KeyMapping.Category CATEGORY = KeyMapping.Category.register(
        Identifier.fromNamespaceAndPath("vlox", "main")
    );

    @Override
    public void onInitializeClient() {
        KeyMapping openMenu = KeyMappingHelper.registerKeyMapping(new KeyMapping(
            "key.vlox.open_menu",
            InputConstants.Type.KEYSYM,
            GLFW.GLFW_KEY_RIGHT_SHIFT,
            CATEGORY
        ));

        HudElementRegistry.attachElementBefore(
            VanillaHudElements.CHAT,
            Identifier.fromNamespaceAndPath("vlox", "client_hud"),
            VloxClient::renderHud
        );

        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            while (openMenu.consumeClick()) {
                if (client.screen instanceof VloxScreen) {
                    client.setScreen(null);
                } else {
                    client.setScreen(new VloxScreen());
                }
            }
        });
    }

    private static void renderHud(GuiGraphicsExtractor graphics, net.minecraft.client.DeltaTracker tickCounter) {
        Minecraft client = Minecraft.getInstance();
        if (client.player == null || client.screen != null) return;

        int y = 8;
        if (showFps) {
            graphics.text(client.font, "VLOX  |  " + client.getFps() + " FPS", 8, y, 0xFF75A8FF, true);
            y += 12;
        }
        if (showCoordinates) {
            graphics.text(client.font, "XYZ  " + client.player.blockPosition().toShortString(), 8, y, 0xFFEAF1FC, true);
        }
    }
}
