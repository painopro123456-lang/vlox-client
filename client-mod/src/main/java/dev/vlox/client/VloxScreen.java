package dev.vlox.client;

import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

public final class VloxScreen extends Screen {
    private final String section;
    private int left;
    private int top;

    public VloxScreen() {
        this("Home");
    }

    private VloxScreen(String section) {
        super(Component.literal("VLOX CLIENT"));
        this.section = section;
    }

    @Override
    protected void init() {
        left = (width - 560) / 2;
        top = (height - 360) / 2;
        int x = left + 18;
        int y = top + 72;

        addRenderableWidget(navButton("Home", "Home", x, y));
        addRenderableWidget(navButton("HUD", "HUD", x, y + 34));
        addRenderableWidget(navButton("Modules", "Modules", x, y + 68));
        addRenderableWidget(navButton("Settings", "Settings", x, y + 102));
        addRenderableWidget(Button.builder(Component.literal("Close"), button -> onClose())
            .bounds(left + 472, top + 320, 70, 24).build());

        if (section.equals("HUD")) {
            addRenderableWidget(toggleButton("FPS counter", true, left + 170, top + 112));
            addRenderableWidget(toggleButton("Coordinates", false, left + 170, top + 148));
        }
    }

    private Button navButton(String label, String target, int x, int y) {
        return Button.builder(Component.literal(label), button -> minecraft.setScreen(new VloxScreen(target)))
            .bounds(x, y, 112, 26).build();
    }

    private Button toggleButton(String label, boolean fps, int x, int y) {
        boolean enabled = fps ? VloxClient.showFps : VloxClient.showCoordinates;
        return Button.builder(Component.literal(label + "   " + (enabled ? "ON" : "OFF")), button -> {
            if (fps) VloxClient.showFps = !VloxClient.showFps;
            else VloxClient.showCoordinates = !VloxClient.showCoordinates;
            minecraft.setScreen(new VloxScreen("HUD"));
        }).bounds(x, y, 220, 28).build();
    }

    @Override
    public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float delta) {
        super.extractRenderState(graphics, mouseX, mouseY, delta);
        graphics.fill(left, top, left + 560, top + 360, 0xF20D1422);
        graphics.fill(left, top, left + 560, top + 54, 0xFF131F32);
        graphics.fill(left, top + 54, left + 145, top + 360, 0xFF101927);
        graphics.outline(left, top, 560, 360, 0xFF344762);
        graphics.text(font, "VLOX", left + 18, top + 17, 0xFF6FA5FF, true);
        graphics.text(font, "CLIENT", left + 66, top + 19, 0xFFEAF1FC, true);
        graphics.text(font, "A cleaner Minecraft experience", left + 170, top + 22, 0xFF9EADC2, false);
        graphics.text(font, section.toUpperCase(), left + 170, top + 82, 0xFFEAF1FC, true);

        if (section.equals("Home")) {
            graphics.text(font, "Welcome to VLOX.", left + 170, top + 116, 0xFFFFFFFF, true);
            graphics.text(font, "Your client. Your settings. Your playstyle.", left + 170, top + 140, 0xFF9EADC2, false);
            graphics.text(font, "Right Shift  |  Open / close menu", left + 170, top + 178, 0xFF6FA5FF, false);
        } else if (section.equals("HUD")) {
            graphics.text(font, "Choose the HUD elements you want to see.", left + 170, top + 98, 0xFF9EADC2, false);
        } else if (section.equals("Modules")) {
            graphics.text(font, "More client modules are being built for VLOX.", left + 170, top + 98, 0xFF9EADC2, false);
        } else {
            graphics.text(font, "VLOX Client  |  Minecraft 26.2", left + 170, top + 116, 0xFF9EADC2, false);
            graphics.text(font, "More settings are coming in later builds.", left + 170, top + 140, 0xFF9EADC2, false);
        }
    }

    @Override
    public void onClose() {
        if (minecraft != null) minecraft.setScreen(null);
    }
}
