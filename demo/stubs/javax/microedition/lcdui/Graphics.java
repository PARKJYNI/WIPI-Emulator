package javax.microedition.lcdui;

public class Graphics {
    public static final int TOP = 16;
    public static final int LEFT = 4;
    public static final int HCENTER = 1;
    public static final int BOTTOM = 32;

    public void setColor(int rgb) {}
    public void fillRect(int x, int y, int w, int h) {}
    public void drawRect(int x, int y, int w, int h) {}
    public void fillArc(int x, int y, int w, int h, int start, int arc) {}
    public void drawString(String s, int x, int y, int anchor) {}
    public void drawLine(int x1, int y1, int x2, int y2) {}
    public void fillRoundRect(int x, int y, int w, int h, int aw, int ah) {}
}
