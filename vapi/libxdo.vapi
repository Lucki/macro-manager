[CCode(cheader_filename = "xdo.h")]
namespace Xdo {
	public const int SUCCESS;
	public const int ERROR;

	[CCode(cname = "charcodemap_t", has_type_id = false)]
	public struct Charcodemap {
		/** the letter for this key, like 'a' */
		char key;
		/** the keycode that this key is on */
		X.KeyCode code;
		/** the symbol representing this key */
		X.KeySym symbol;
		/** the keyboard group that has this key in it */
		int group;
		/** the modifiers to apply when sending this key */
		int modmask;
		/** if this key need to be bound at runtime because it does not
		 * exist in the current keymap, this will be set to 1. */
		int needs_binding;
	}

	[Compact]
	[CCode(cname = "xdo_t", cprefix = "xdo_", destroy_function = "xdo_free", has_type_id = false)]
	public class Xdo {
		/**
		 * Create a new xdo_t instance.
		 *
		 * @param display the string display name, such as ":0". If null, uses the
		 * environment variable DISPLAY just like XOpenDisplay(NULL).
		 *
		 * @return Pointer to a new xdo_t or NULL on failure
		 */
		public static Xdo? new(string? display = null);

		/** The Display for Xlib */
		X.Display? xdpy;

		/** The display name, if any. NULL if not specified. */
		string? display_name;

		/** @internal Array of known keys/characters */
		private Charcodemap[]? charcodes;

		/** @internal Length of charcodes array */
		private int charcodes_len;

		/** @internal highest keycode value */
		/* highest and lowest keycodes */
		/* used by this X server */
		private int keycode_high;

		/** @internal lowest keycode value */
		/* highest and lowest keycodes */
		/* used by this X server */
		private int keycode_low;

		/** @internal number of keysyms per keycode */
		private int keysyms_per_keycode;

		/** Should we close the display when calling xdo_free? */
		int close_display_when_freed;

		/** Be extra quiet? (omits some error/message output) */
		int quiet;

		/** Enable debug output? */
		int debug;

		/** Feature flags, such as XDO_FEATURE_XTEST, etc... */
		int features_mask;

		/**
		 * Get the window currently having focus.
		 *
		 * @param window Pointer to a window where the currently-focused window
		 *   will be stored.
		 */
		public int get_focused_window(out X.Window window);

		/**
		 * Like xdo_get_focused_window, but return the first ancestor-or-self window *
		 * having a property of WM_CLASS. This allows you to get the "real" or
		 * top-level-ish window having focus rather than something you may not expect
		 * to be the window having focused.
		 *
		 * @param window Pointer to a window where the currently-focused window
		 *   will be stored.
		 */
		public int get_focused_window_sane(out X.Window window);

		/**
		 * Get a window's size.
		 *
		 * @param wid the window to query
		 * @param width pointer to unsigned int where the width is stored.
		 * @param height pointer to unsigned int where the height is stored.
		 */
		public int get_window_size(X.Window window, out uint width, out uint height);

		/**
		 * Get the PID owning a window. Not all applications support this.
		 * It looks at the _NET_WM_PID property of the window.
		 *
		 * @param window the window to query.
		 * @return the process id or 0 if no pid found.
		 */
		public int get_pid_window(X.Window window);

		/**
		 * Get the current mouse location (coordinates and screen number).
		 *
		 * @param x integer pointer where the X coordinate will be stored
		 * @param y integer pointer where the Y coordinate will be stored
		 * @param screen_num integer pointer where the screen number will be stored
		 */
		public int get_mouse_location(out int x, out int y, out int screen_num);

		/**
		 * Get a window's location.
		 *
		 * @param wid the window to query
		 * @param x_ret pointer to int where the X location is stored. If NULL, X is
		 *   ignored.
		 * @param y_ret pointer to int where the Y location is stored. If NULL, X is
		 *   ignored.
		 * @param screen_ret Pointer to Screen* where the Screen* the window on is
		 *   stored. If NULL, this parameter is ignored.
		 */
		public int get_window_location(X.Window window, ref int? x, ref int? y, ref X.Screen? screen);
	}
}
