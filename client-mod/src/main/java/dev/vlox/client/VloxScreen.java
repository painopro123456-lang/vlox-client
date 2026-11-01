package dev.vlox.client;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.text.Text;

public final class VloxScreen extends Screen {
    private String section = "Home";
    private int left;
    private int top;

    public VloxScreen() { super(Text.literal("VLOX CLIENT")); }

    @Override
    protected void init() {
        left = (width - 560) / 2;
        top = (height - 360) / 2;
        int x = left + 18, y = top + 72;
        addDrawableChild(ButtonWidget.builder(Text.literal("Home"), b -> change("Home")).dimensions(x, y, 112, 26).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("HUD"), b -> change("HUD")).dimensions(x, y + 34, 112, 26).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Modules"), b -> change("Modules")).dimensions(x, y + 68, 112, 26).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Settings"), b -> change("Settings")).dimensions(x, y + 102, 112, 26).build());
        addDrawableChild(ButtonWidget.builder(Text.literal("Close"), b -> close()).dimensions(left + 472, top + 320, 70, 24).build());
        if (section.equals("HUD")) {
            addToggle("FPS counter", VloxClient.showFps, v -> VloxClient.showFps = v, left + 170, top + 112);
            addToggle("Coordinates", VloxClient.showCoordinates, v -> VloxClient.showCoordinates = v, left + 170, top + 148);
        } else if (section.equals("Modules")) {
            addToggle("Clean chat", VloxClient.cleanChat, v -> VloxClient.cleanChat = v, left + 170, top + 112);
        }
    }

    private void change(String next) { section = next; clearAndInit(); }
    private void addToggle(String label, boolean enabled, java.util.function.Consumer<Boolean> setter, int x, int y) {
        addDrawableChild(ButtonWidget.builder(Text.literal(label + "   " + (enabled ? "ON" : "OFF")), b -> {
            setter.accept(!enabled);
            clearAndInit();
        }).dimensions(x, y, 220, 28).build());
    }

    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        renderBackground(context, mouseX, mouseY, delta);
        context.fill(left, top, left + 560, top + 360, 0xF20D1422);
        context.fill(left, top, left + 560, top + 54, 0xFF131F32);
        context.fill(left, top + 54, left + 145, top + 360, 0xFF101927);
        context.drawBorder(left, top, 560, 360, 0xFF344762);
        context.drawText(textRenderer, "VLOX", left + 18, top + 17, 0xFF6FA5FF, true);
        context.drawText(textRenderer, "CLIENT", left + 66, top + 19, 0xFFEAF1FC, true);
        context.drawText(textRenderer, "A cleaner Minecraft experience", left + 170, top + 22, 0xFF9EADC2, false);
        context.drawText(textRenderer, section.toUpperCase(), left + 170, top + 82, 0xFFEAF1FC, true);
        if (section.equals("Home")) {
            context.drawText(textRenderer, "Welcome to VLOX.", left + 170, top + 116, 0xFFFFFFFF, true);
            context.drawText(textRenderer, "Your client. Your settings. Your playstyle.", left + 170, top + 140, 0xFF9EADC2, false);
            context.drawText(textRenderer, "Right Shift  •  Open / close menu", left + 170, top + 178, 0xFF6FA5FF, false);
        } else if (section.equals("HUD")) {
            context.drawText(textRenderer, "Choose the HUD elements you want to see.", left + 170, top + 98, 0xFF9EADC2, false);
        } else if (section.equals("Modules")) {
            context.drawText(textRenderer, "Client-side quality-of-life options.", left + 170, top + 98, 0xFF9EADC2, false);
        } else {
            context.drawText(textRenderer, "VLOX Client • Fabric 1.21.1", left + 170, top + 116, 0xFF9EADC2, false);
            context.drawText(textRenderer, "More settings are coming in later builds.", left + 170, top + 140, 0xFF9EADC2, false);
        }
        super.render(context, mouseX, mouseY, delta);
    }

    @Override public boolean shouldPause() { return false; }
    @Override public void close() { if (client != null) client.setScreen(null); }
}
