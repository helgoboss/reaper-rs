// This is a stripped-down version of the official "swell-modstub.mm".
//
// See "swell-modstub-generic-custom.cpp" for details.

#import <Cocoa/Cocoa.h>
#import <objc/objc-runtime.h>
#define SWELL_API_DEFPARM(x)
#define SWELL_API_DEFINE(ret,func,parms) ret (*func) parms ;
#include "../lib/WDL/WDL/swell/swell.h"

// only include this file in projects that are linked to swell.dylib

struct SWELL_DialogResourceIndex *SWELL_curmodule_dialogresource_head;
struct SWELL_MenuResourceIndex *SWELL_curmodule_menuresource_head;

// define the functions

static struct
{
  const char *name;
  void **func;
} api_tab[]={
  
#undef _WDL_SWELL_H_API_DEFINED_
#undef SWELL_API_DEFINE
#define SWELL_API_DEFINE(ret, func, parms) {#func, (void **)&func },

#include "../lib/WDL/WDL/swell/swell-functions.h"
  
};

static int dummyFunc() { return 0; }

class SwellAPPInitializer
{
public:
  SwellAPPInitializer()
  {
    void *(*SWELLAPI_GetFunc)(const char *name)=NULL;
    void *(*send_msg)(id, SEL) = (void *(*)(id, SEL))objc_msgSend;
    
    id del = [NSApp delegate];
    if (del && [del respondsToSelector:@selector(swellGetAPPAPIFunc)])
      *(void **)&SWELLAPI_GetFunc = send_msg(del,@selector(swellGetAPPAPIFunc));

    if (!SWELLAPI_GetFunc) NSLog(@"SWELL API provider not found\n");
    else if (SWELLAPI_GetFunc(NULL)!=(void*)0x100)
    {
      NSLog(@"SWELL API provider returned incorrect version\n");
      SWELLAPI_GetFunc=0;
    }
      
    int x;
    for (x = 0; x < sizeof(api_tab)/sizeof(api_tab[0]); x ++)
    {
      *api_tab[x].func=SWELLAPI_GetFunc?SWELLAPI_GetFunc(api_tab[x].name):0;
      if (!*api_tab[x].func)
      {
        if (SWELLAPI_GetFunc) NSLog(@"SWELL API not found: %s\n",api_tab[x].name);
        *api_tab[x].func = (void*)&dummyFunc;
      }
    }
  }
  ~SwellAPPInitializer()
  {
  }
};

SwellAPPInitializer m_swell_appAPIinit;

extern "C" __attribute__ ((visibility ("default"))) int SWELL_dllMain(HINSTANCE hInst, DWORD callMode, LPVOID _GetFunc)
{
  // this returning 1 allows DllMain to be called, if available
  return 1;
}
