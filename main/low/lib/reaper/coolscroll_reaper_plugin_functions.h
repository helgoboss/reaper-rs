// This file contains function definitions for WDL's coolscroll, which is exposed by REAPER as well.
// This is copied from `coolscroll.h` and turned into function pointers (which are detected by our Rust code generator).

BOOL	WINAPI (*InitializeCoolSB)(HWND hwnd);
HRESULT WINAPI (*UninitializeCoolSB)(HWND hwnd); // call in WM_DESTROY -- not strictly required, but recommended

BOOL WINAPI (*CoolSB_SetMinThumbSize)(HWND hwnd, UINT wBar, UINT size);
//BOOL WINAPI (*CoolSB_IsThumbTracking)(HWND hwnd);
//BOOL WINAPI (*CoolSB_IsCoolScrollEnabled)(HWND hwnd);
void CoolSB_SetVScrollPad(HWND hwnd, UINT topamt, UINT botamt, void *(*getDeadAreaBitmap)(int which, HWND hwnd, RECT *, int defcol));
//
BOOL WINAPI (*CoolSB_GetScrollInfo)(HWND hwnd, int fnBar, LPSCROLLINFO lpsi);
//int	 WINAPI (*CoolSB_GetScrollPos)(HWND hwnd, int nBar);
//BOOL WINAPI (*CoolSB_GetScrollRange)(HWND hwnd, int nBar, LPINT lpMinPos, LPINT lpMaxPos);

//
int	 WINAPI (*CoolSB_SetScrollInfo	)(HWND hwnd, int fnBar, LPSCROLLINFO lpsi, BOOL fRedraw);
int  WINAPI (*CoolSB_SetScrollPos	)(HWND hwnd, int nBar, int nPos, BOOL fRedraw);
int  WINAPI (*CoolSB_SetScrollRange	)(HWND hwnd, int nBar, int nMinPos, int nMaxPos, BOOL fRedraw);
BOOL WINAPI (*CoolSB_ShowScrollBar	)(HWND hwnd, int wBar, BOOL fShow);

BOOL WINAPI (*CoolSB_SetResizingThumb)(HWND hwnd, BOOL active);
BOOL WINAPI (*CoolSB_SetThemeIndex)(HWND hwnd, int idx);
//void (*CoolSB_SetScale)(float scale); // sets scale to use for scrollbars (does not refresh, though -- set this at startup/etc)
//void (*CoolSB_OnColorThemeChange)(); // refreshes all