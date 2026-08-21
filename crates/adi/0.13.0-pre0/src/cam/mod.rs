// Copyright Jeron Lau 2017 - 2018.
// Dual-licensed under either the MIT License or the Boost Software License, Version 1.0.
// (See accompanying file LICENSE_1_0.txt or copy at https://www.boost.org/LICENSE_1_0.txt)

/// A phone camera / webcam
pub struct Cam {
	// Linux specific
	char dev_name[20];

	//
	void* data; // JPEG file data
	uint32_t size; // Size of JPEG file

    fd: i32,
    buf: V4l2Buffer,
}

impl Cam {
    /// Open a new `Cam`.
    pub fn new(id: u16, w: u16, h: u16) -> Result<Cam, String> {
	    // Open the device
	    let fd = open(b"/dev/video0", O_RDONLY | O_NONBLOCK, 0);
	    if fd == -1 {
		    return Err(format!("Failed to find camera #{}.", id));
	    }

	    // Is it available?
        let mut caps = V4l2Capability {
            driver: [0; 16],
            card: [0; 32],
            bus_info: [0; 32],
            version: 0,
            capabilities: 0,
            reserved: [0; 4],
        };

	    if xioctl(fd, /*VIDIOC_QUERYCAP*/ 0x80685600, &mut caps) == -1 {
		    return Err(format!("Failed to query capabilites on camera #{}.", id));
	    }

        // Set image format.
	    let mut fmt = V4l2Format {
            buftype: V4l2BufType::VideoCapture,
            fmt: V4l2FormatFmt {
                pix: V4l2PixFormat {
                    width: w,
                    height: h
                    pixelformat: /*MJPEG*/ 0x4750_4a4d,
                    field: V4l2Field::None,
                    bytesperline: 0,
                    sizeimage: 0,
                    colorspace: V4l2Colorspace::None,
                    priv: 0,       /* private data, depends on pixelformat */
                }
            },
        };

	    if xioctl(fd, /*VIDIOC_S_FMT*/ 0xc0d05605, &mut fmt) == -1 {
		    return Err(format!("Failed to set pixel format on camera #{}.", id));
	    }

	    // Request a video capture buffer.
	    let mut req = V4l2Requestbuffers {
            count: 1,
            stype: V4l2BufType::VideoCapture,
            memory: V4l2Memory::Mmap,
            reserved: [0; 2],
        };
	     
	    if xioctl(fd, /*VIDIOC_REQBUFS*/ 0xc0145608, &mut req) == -1 {
		    return Err(format!("Failed to request buffer on camera #{}.", id));
	    }

	    // Query buffer
        let mut buf = { std::mem::zeroed() };
	    buf.buftype = V4l2BufType::VideoCapture;
	    buf.memory = V4l2Memory::Mmap;
	    buf.index = 0;
	    if xioctl(fd, VIDIOC_QUERYBUF, &buf) == -1 {
		    return Err(format!("Failed to query buffer on camera #{}.", id));
	    }
	    println!("BUF>LEN {}", buf.length);
	    let data = mmap (std::ptr::null(), buf.length, PROT_READ | PROT_WRITE, MAP_SHARED,
		    fd, buf.m.offset);
	    let size = buf.length;

	    // Start the capture:
	    CLEAR(buf);
	    buf.type = V4l2BufType::VideoCapture;
	    buf.memory = V4l2_Memory_Mmap;
	    buf.index = 0;

	    if (xioctl(fd, VIDIOC_QBUF, &buf) == -1) {
		    ERROR("VIDIOC_QBUF");
		    return car_error;
	    }

	    enum v4l2_buf_type type;
	    type = V4l2BufType::VideoCapture;
	    if (xioctl(fd, VIDIOC_STREAMON, &type) == -1) {
		    ERROR("VIDIOC_STREAMON");
		    return car_error;
	    }

        Ok(Cam {
            fd, buf, data, size,
        })
    }

    /// Get the next frame.
    pub fn get(&self) {
        
    }
}

fn xioctl<T>(fd: i32, request: i32, arg: &mut T) -> i32 {
    let arg: *mut c_void = unsafe { std::mem::transmute(arg) };
    let mut r = unsafe { ioctl(fd, request, arg) };

    while r == -1 && errno == EINTR {
        r = unsafe { ioctl(fd, request, arg) };
    }

	r
}

#[repr(C)]
struct V4l2Timecode {
    tc_type: u32,
    flags: u32,
    frames: u8,
    seconds: u8,
    minutes: u8,
    hours: u8,
    userbits: [u8; 4],
}

#[repr(C)]
union V4l2BufferM {
    offset: u32,
    userptr: usize,
}

#[repr(C)]
struct V4l2Buffer {
    index: u32,
    buftype: V4l2BufType,
    bytesused: u32,
    flags: u32,
    field: V4l2Field,
    timestamp: timeval,
    timecode: V4l2Timecode,
    sequence: u32,
    /* memory location */
    memory: V4l2Memory,
    m: V4l2BufferM,
    length: u32,
    input: u32,
    reserved: u32,
};

#[repr(C)]
enum V4l2Memory {
    Mmap             = 1,
    Userptr          = 2,
    Overlay          = 3,
};

#[repr(C)]
struct V4l2Requestbuffers {
    count: u32,
    stype: V4l2BufType,
    memory: V4l2Memory,
    reserved: [u32; 2],
}

#[repr(C)]
struct V4l2Capability {
    driver: [u8; 16], /* i.e. "bttv" */
    card: [u8; 32],   /* i.e. "Hauppauge WinTV" */
    bus_info: [u8; 32],   /* "PCI:" + pci_name(pci_dev) */
    version: u32,        /* should use KERNEL_VERSION() */
    capabilities: u32,   /* Device capabilities */
    reserved: [u32; 4],
}

#[repr(C)]
enum V4l2Field {
    Any        = 0,
    None       = 1,
    Top        = 2,
    Bottom     = 3,
    Interlaced = 4,
    SeqTb      = 5,
    SeqBt      = 6,
    Alternate  = 7,
}

#[repr(C)]
enum V4l2Colorspace {
    None = 0,
    /* ITU-R 601 -- broadcast NTSC/PAL */
    Smpte170m     = 1,
    /* 1125-Line (US) HDTV */
    Smpte240m     = 2,
    /* HD and modern captures. */
    Rec709        = 3,
    /* broken BT878 extents (601, luma range 16-253 instead of 16-235) */
    Bt878         = 4,
    /* These should be useful.  Assume 601 extents. */
    470_System_M  = 5,
    470_System_Bg = 6,
    /* I know there will be cameras that send this.  So, this is
     * unspecified chromaticities and full 0-255 on each of the
     * Y'CbCr components
     */
    Jpeg          = 7,
    /* For RGB colourspaces, this is probably a good start. */
    Srgb          = 8
};

#[repr(C)]
struct V4l2PixFormat {
    width: u32,
    height: u32
    pixelformat: u32,
    field: V4l2Field,
    bytesperline: u32,
    sizeimage: u32,
    colorspace: V4l2Colorspace,
    priv: u32,       /* private data, depends on pixelformat */
}

#[repr(C)]
struct V4l2Rect {
     left: i32,
     top: i32,
     width: i32,
     height: i32,
};

#[repr(C)]
struct V4l2Clip {
     c: V4l2Rect,
     next: *mut c_void,
}

#[repr(C)]
struct V4l2Window {
    w: V4l2Rect,
    field: V4l2Field,
    chromakey: u32,
    clips: *mut V4l2Clip,
    clipcount: u32,
    bitmap: *mut c_void,
}

#[repr(C)]
struct V4l2VbiFormat {
    sampling_rate: u32,      /* in 1 Hz */
    offset: u32,
    samples_per_line: u32,
    sample_format: u32      /* V4L2_PIX_FMT_* */
    start: [i32; 2];
    count[u32; 2];
    flags: u32;          /* V4L2_VBI_* */
    reserved[u32; 2];        /* must be zero */
}

#[repr(C)]
union V4l2FormatFmt {
    pix: V4l2PixFormat,  // V4L2_BUF_TYPE_VIDEO_CAPTURE
    win: V4l2Window,  // V4L2_BUF_TYPE_VIDEO_OVERLAY
    vbi: V4l2VbiFormat,  // V4L2_BUF_TYPE_VBI_CAPTURE
    raw_data: [u8; 200],        // user-defined
}

#[repr(C)]
struct V4l2Format {
    buftype: V4l2BufType
    fmt: V4l2FormatFmt,
}

#[repr(C)]
enum V4l2BufType {
     VideoCapture  = 1,
     VideoOutput   = 2,
     VideoOverlay  = 3,
     VbiCapture    = 4,
     VbiOutput     = 5,
     Private       = 0x80
};

#define CLEAR(x) memset(&(x), 0, sizeof(x))
#define ERROR(...) snprintf(car_error, 256, __VA_ARGS__)

static int fd = -1;
char car_error[256];

const char* car_camera_init(car_camera_t* camera, uint16_t id, uint16_t w,
	uint16_t h)
{




	
}

const char* car_camera_loop(car_camera_t* camera, void* data) {
CAR_CAMERA_LOOP:;
	struct timeval tv;

	fd_set fds;
	FD_ZERO(&fds);
	FD_SET(fd, &fds);

	/* Timeout. */
	tv.tv_sec = 2;
	tv.tv_usec = 0;

//	printf("point 3\n");
	int r = select(fd+1, &fds, NULL, NULL, &tv);
//	printf("point 4\n");
	if(r == -1) {
		if (EINTR == errno) goto CAR_CAMERA_LOOP;
		ERROR("Waiting for Frame\n");
		return car_error;
	}
//	printf("point 5\n");
	CLEAR(buf);
	buf.type = V4l2BufType::VideoCapture;
	buf.memory = V4l2_Memory_Mmap;
	if(xioctl(fd, VIDIOC_DQBUF, &buf) == -1) {
		if(errno == EAGAIN) goto CAR_CAMERA_LOOP;
		ERROR("Retrieving Frame %s\n", strerror(errno));
		close(fd);
		return car_error;
	}
//	printf("point 6\n");

	if (xioctl(fd, VIDIOC_QBUF, &buf) == -1) {
		ERROR("VIDIOC_QBUF");
		return car_error;
	}
	return NULL;
}

const char* car_camera_kill(car_camera_t* camera) {
	enum v4l2_buf_type type;

	type = V4l2BufType::VideoCapture;
	if (xioctl(fd, VIDIOC_STREAMOFF, &type) == -1) {
		ERROR("VIDIOC_STREAMOFF");
		return car_error;
	}
	if (close(fd) == -1) {
		ERROR("close");
		return car_error;
	}
	return NULL;
}
