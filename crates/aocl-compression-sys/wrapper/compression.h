/* AOCL-Compression umbrella header.
 *
 * The unified API entrypoint is aocl_compression.h; per-algorithm headers
 * (lz4.h, lz4hc.h, lz4frame.h, snappy-c.h, bzlib.h, LzmaEnc.h, LzmaDec.h)
 * are also pulled in for direct algorithm-specific access.
 */
#include <aocl_compression.h>
#include <lz4.h>
#include <lz4hc.h>
/* AOCL 5.3.x (Windows) ships lz4.h and lz4hc.h but not lz4frame.h; include the frame API only where it exists. */
#if defined(__has_include)
#  if __has_include(<lz4frame.h>)
#    include <lz4frame.h>
#  endif
#else
#  include <lz4frame.h>
#endif
#include <snappy-c.h>
#include <bzlib.h>
#include <LzmaEnc.h>
#include <LzmaDec.h>
