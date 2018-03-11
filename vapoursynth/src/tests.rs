#![cfg(test)]
use super::*;

// We need the VSScript functions, and either VSScript API 3.2 or the VapourSynth functions.
#[cfg(all(feature = "vsscript-functions",
          any(feature = "vapoursynth-functions", feature = "gte-vsscript-api-32")))]
mod need_api_and_vsscript {
    use std::sync::mpsc::channel;

    use super::*;
    use prelude::*;
    use video_info::{Framerate, Resolution};

    fn props_test(frame: &Frame, fps_num: i64) {
        let props = frame.props();
        assert_eq!(props.key_count(), 2);
        assert_eq!(props.key(0), "_DurationDen");
        assert_eq!(props.key(1), "_DurationNum");

        assert_eq!(props.value_count(props.key(0)), Ok(1));
        assert_eq!(props.get_int(props.key(0)), Ok(fps_num));
        assert_eq!(props.value_count(props.key(1)), Ok(1));
        assert_eq!(props.get_int(props.key(1)), Ok(1));
    }

    fn env_video_var_test(env: &vsscript::Environment) {
        let mut map = OwnedMap::new(API::get().unwrap());
        assert!(env.get_variable("video", &mut map).is_ok());
        assert_eq!(map.key_count(), 1);
        assert_eq!(map.key(0), "video");
        let node = map.get_node("video");
        assert!(node.is_ok());
    }

    fn green_frame_test(frame: &Frame) {
        let format = frame.format();
        assert_eq!(format.name(), "RGB24");
        assert_eq!(format.plane_count(), 3);

        for plane in 0..format.plane_count() {
            let resolution = frame.resolution(plane);
            assert_eq!(
                resolution,
                Resolution {
                    width: 1920,
                    height: 1080,
                }
            );

            let color = if plane == 1 { [255; 1920] } else { [0; 1920] };

            for row in 0..resolution.height {
                let data_row = frame.data_row(plane, row);
                assert_eq!(&data_row[..], &color[..]);
            }
        }
    }

    fn green_test(env: &vsscript::Environment) {
        #[cfg(feature = "gte-vsscript-api-31")]
        let (node, alpha_node) = {
            let output = env.get_output(0);
            assert!(output.is_ok());
            output.unwrap()
        };
        #[cfg(not(feature = "gte-vsscript-api-31"))]
        let (node, alpha_node) = (env.get_output(0).unwrap(), None::<Node>);

        assert!(alpha_node.is_none());

        let info = node.info();

        if let Property::Constant(format) = info.format {
            assert_eq!(format.name(), "RGB24");
        } else {
            assert!(false);
        }

        assert_eq!(
            info.framerate,
            Property::Constant(Framerate {
                numerator: 60,
                denominator: 1,
            })
        );
        assert_eq!(
            info.resolution,
            Property::Constant(Resolution {
                width: 1920,
                height: 1080,
            })
        );

        #[cfg(feature = "gte-vapoursynth-api-32")]
        assert_eq!(info.num_frames, 100);
        #[cfg(not(feature = "gte-vapoursynth-api-32"))]
        assert_eq!(info.num_frames, Property::Constant(100));

        let frame = node.get_frame(0).unwrap();
        green_frame_test(&frame);
        props_test(&frame, 60);
        env_video_var_test(&env);
    }

    #[test]
    fn green() {
        let env =
            vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();
        green_test(&env);
    }

    #[test]
    fn green_from_string() {
        let env =
            vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();
        green_test(&env);
    }

    #[test]
    fn variable() {
        let env =
            vsscript::Environment::from_file("test-vpy/variable.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();

        #[cfg(feature = "gte-vsscript-api-31")]
        let (node, alpha_node) = {
            let output = env.get_output(0);
            assert!(output.is_ok());
            output.unwrap()
        };
        #[cfg(not(feature = "gte-vsscript-api-31"))]
        let (node, alpha_node) = (env.get_output(0).unwrap(), None::<Node>);

        assert!(alpha_node.is_none());

        let info = node.info();

        assert_eq!(info.format, Property::Variable);
        assert_eq!(info.framerate, Property::Variable);
        assert_eq!(info.resolution, Property::Variable);

        #[cfg(feature = "gte-vapoursynth-api-32")]
        assert_eq!(info.num_frames, 200);
        #[cfg(not(feature = "gte-vapoursynth-api-32"))]
        assert_eq!(info.num_frames, Property::Constant(200));

        // Test the first frame.
        let frame = node.get_frame(0).unwrap();
        let format = frame.format();
        assert_eq!(format.name(), "RGB24");
        assert_eq!(format.plane_count(), 3);

        for plane in 0..format.plane_count() {
            let resolution = frame.resolution(plane);
            assert_eq!(
                resolution,
                Resolution {
                    width: 1920,
                    height: 1080,
                }
            );

            let color = if plane == 1 { [255; 1920] } else { [0; 1920] };

            for row in 0..resolution.height {
                let data_row = frame.data_row(plane, row);
                assert_eq!(&data_row[..], &color[..]);
            }
        }

        props_test(&frame, 60);

        // Test the first frame of the next format.
        let frame = node.get_frame(100).unwrap();
        let format = frame.format();
        assert_eq!(format.name(), "Gray8");
        assert_eq!(format.plane_count(), 1);

        let plane = 0;
        let resolution = frame.resolution(plane);
        assert_eq!(
            resolution,
            Resolution {
                width: 1280,
                height: 720,
            }
        );

        let color = [127; 1280];

        for row in 0..resolution.height {
            let data_row = frame.data_row(plane, row);
            assert_eq!(&data_row[..], &color[..]);
        }

        props_test(&frame, 30);
        env_video_var_test(&env);
    }

    #[test]
    #[cfg(feature = "gte-vsscript-api-31")]
    fn alpha() {
        let env =
            vsscript::Environment::from_file("test-vpy/alpha.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();

        let (_, alpha_node) = {
            let output = env.get_output(0);
            assert!(output.is_ok());
            output.unwrap()
        };

        assert!(alpha_node.is_some());
        let alpha_node = alpha_node.unwrap();

        let info = alpha_node.info();

        if let Property::Constant(format) = info.format {
            assert_eq!(format.name(), "Gray8");
        } else {
            assert!(false);
        }

        assert_eq!(
            info.framerate,
            Property::Constant(Framerate {
                numerator: 60,
                denominator: 1,
            })
        );
        assert_eq!(
            info.resolution,
            Property::Constant(Resolution {
                width: 1920,
                height: 1080,
            })
        );

        #[cfg(feature = "gte-vapoursynth-api-32")]
        assert_eq!(info.num_frames, 100);
        #[cfg(not(feature = "gte-vapoursynth-api-32"))]
        assert_eq!(info.num_frames, Property::Constant(100));

        let frame = alpha_node.get_frame(0).unwrap();
        let format = frame.format();
        assert_eq!(format.name(), "Gray8");
        assert_eq!(format.plane_count(), 1);

        let resolution = frame.resolution(0);
        assert_eq!(
            resolution,
            Resolution {
                width: 1920,
                height: 1080,
            }
        );

        for row in 0..resolution.height {
            let data_row = frame.data_row(0, row);
            assert_eq!(&data_row[..], &[128; 1920][..]);
        }
    }

    #[test]
    fn clear_output() {
        let env =
            vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();
        assert!(
            env.clear_output(1)
                .err()
                .map(|e| if let vsscript::Error::NoOutput = e {
                    true
                } else {
                    false
                })
                .unwrap_or(false)
        );
        assert!(env.clear_output(0).is_ok());
        assert!(
            env.clear_output(0)
                .err()
                .map(|e| if let vsscript::Error::NoOutput = e {
                    true
                } else {
                    false
                })
                .unwrap_or(false)
        );
    }

    #[test]
    fn iterators() {
        let env =
            vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();

        #[cfg(feature = "gte-vsscript-api-31")]
        let (node, alpha_node) = env.get_output(0).unwrap();
        #[cfg(not(feature = "gte-vsscript-api-31"))]
        let (node, alpha_node) = (env.get_output(0).unwrap(), None::<Node>);

        assert!(alpha_node.is_none());

        let frame = node.get_frame(0).unwrap();
        let props = frame.props();

        assert_eq!(props.keys().size_hint(), (2, Some(2)));
        // assert_eq!(props.iter().size_hint(), (2, Some(2)));
    }

    #[test]
    fn vsscript_variables() {
        let env =
            vsscript::Environment::from_script(include_str!("../test-vpy/green.vpy")).unwrap();

        let mut map = OwnedMap::new(API::get().unwrap());
        assert!(env.get_variable("video", &mut map).is_ok());
        assert!(env.clear_variable("video").is_ok());
        assert!(env.clear_variable("video").is_err());
        assert!(env.get_variable("video", &mut map).is_err());

        assert!(env.set_variables(&map).is_ok());
        assert!(env.get_variable("video", &mut map).is_ok());
    }

    #[test]
    fn get_frame_async() {
        let env =
            vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();

        #[cfg(feature = "gte-vsscript-api-31")]
        let (node, alpha_node) = {
            let output = env.get_output(0);
            assert!(output.is_ok());
            output.unwrap()
        };
        #[cfg(not(feature = "gte-vsscript-api-31"))]
        let (node, alpha_node) = (env.get_output(0).unwrap(), None::<Node>);

        assert!(alpha_node.is_none());

        let mut rxs = Vec::new();

        for i in 0..10 {
            let (tx, rx) = channel();
            rxs.push(rx);

            node.get_frame_async(i, move |frame, n, node| {
                assert!(frame.is_ok());
                let frame = frame.unwrap();

                assert_eq!(n, i);
                assert_eq!(
                    node.info().framerate,
                    Property::Constant(Framerate {
                        numerator: 60,
                        denominator: 1,
                    })
                );

                green_frame_test(&frame);
                props_test(&frame, 60);

                assert_eq!(tx.send(()), Ok(()));
            });
        }

        drop(node); // Test dropping prematurely.

        for rx in rxs {
            assert_eq!(rx.recv(), Ok(()));
        }
    }

    #[test]
    fn get_frame_async_error() {
        let env =
            vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();

        #[cfg(feature = "gte-vsscript-api-31")]
        let (node, alpha_node) = {
            let output = env.get_output(0);
            assert!(output.is_ok());
            output.unwrap()
        };
        #[cfg(not(feature = "gte-vsscript-api-31"))]
        let (node, alpha_node) = (env.get_output(0).unwrap(), None::<Node>);

        assert!(alpha_node.is_none());

        let (tx, rx) = channel();

        // The clip only has 100 frames, so requesting the 101th one produces an error.
        node.get_frame_async(100, move |frame, n, node| {
            assert!(frame.is_err());
            assert_eq!(n, 100);
            assert_eq!(
                node.info().framerate,
                Property::Constant(Framerate {
                    numerator: 60,
                    denominator: 1,
                })
            );

            assert_eq!(tx.send(()), Ok(()));
        });

        assert_eq!(rx.recv(), Ok(()));
    }

    #[test]
    fn core() {
        let env =
            vsscript::Environment::from_file("test-vpy/green.vpy", vsscript::EvalFlags::Nothing)
                .unwrap();

        let core = env.get_core();
        assert!(core.is_ok());
        let core = core.unwrap();

        let yuv420p8 = core.get_format(PresetFormat::YUV420P8.into());
        assert!(yuv420p8.is_some());
        let yuv420p8 = yuv420p8.unwrap();

        assert_eq!(yuv420p8.id(), PresetFormat::YUV420P8.into());
        assert_eq!(yuv420p8.name(), "YUV420P8");
        assert_eq!(yuv420p8.plane_count(), 3);
        assert_eq!(yuv420p8.color_family(), ColorFamily::YUV);
        assert_eq!(yuv420p8.sample_type(), SampleType::Integer);
        assert_eq!(yuv420p8.bits_per_sample(), 8);
        assert_eq!(yuv420p8.bytes_per_sample(), 1);
        assert_eq!(yuv420p8.sub_sampling_w(), 1);
        assert_eq!(yuv420p8.sub_sampling_h(), 1);

        let yuv422p8 = core.get_format(PresetFormat::YUV422P8.into()).unwrap();
        assert_eq!(yuv422p8.sub_sampling_w(), 1);
        assert_eq!(yuv422p8.sub_sampling_h(), 0);
    }
}

// We need either VSScript API 3.2 or the VapourSynth functions.
#[cfg(any(feature = "vapoursynth-functions",
          all(feature = "vsscript-functions", feature = "gte-vsscript-api-32")))]
mod need_api {
    use std::ffi::CString;
    use std::sync::mpsc::{channel, Sender};
    use std::sync::Mutex;

    use super::*;
    use prelude::*;

    #[test]
    fn maps() {
        let mut map = OwnedMap::new(API::get().unwrap());

        assert_eq!(map.key_count(), 0);

        assert_eq!(map.touch("test_frame", ValueType::Frame), Ok(()));
        assert_eq!(map.value_type("test_frame"), Ok(ValueType::Frame));
        assert_eq!(map.value_count("test_frame"), Ok(0));
        assert_eq!(
            map.append_int("test_frame", 42),
            Err(map::Error::WrongValueType)
        );

        assert_eq!(map.set_int("i", 42), Ok(()));
        assert_eq!(map.get_int("i"), Ok(42));
        assert_eq!(map.append_int("i", 43), Ok(()));
        assert_eq!(map.get_int("i"), Ok(42));
        {
            let iter = map.get_int_iter("i");
            assert!(iter.is_ok());
            let mut iter = iter.unwrap();
            assert_eq!(iter.next(), Some(42));
            assert_eq!(iter.next(), Some(43));
            assert_eq!(iter.next(), None);
        }

        #[cfg(feature = "gte-vapoursynth-api-31")]
        {
            assert_eq!(map.get_int_array("i"), Ok(&[42, 43][..]));

            assert_eq!(map.set_int_array("ia", &[10, 20, 30]), Ok(()));
            assert_eq!(map.get_int_array("ia"), Ok(&[10, 20, 30][..]));
        }

        assert_eq!(map.set_float("f", 42f64), Ok(()));
        assert_eq!(map.get_float("f"), Ok(42f64));
        assert_eq!(map.append_float("f", 43f64), Ok(()));
        assert_eq!(map.get_float("f"), Ok(42f64));
        {
            let iter = map.get_float_iter("f");
            assert!(iter.is_ok());
            let mut iter = iter.unwrap();
            assert_eq!(iter.next(), Some(42f64));
            assert_eq!(iter.next(), Some(43f64));
            assert_eq!(iter.next(), None);
        }

        #[cfg(feature = "gte-vapoursynth-api-31")]
        {
            assert_eq!(map.get_float_array("f"), Ok(&[42f64, 43f64][..]));

            assert_eq!(map.set_float_array("fa", &[10f64, 20f64, 30f64]), Ok(()));
            assert_eq!(map.get_float_array("fa"), Ok(&[10f64, 20f64, 30f64][..]));
        }

        assert_eq!(map.set_data("d", &[1, 2, 3]), Ok(()));
        assert_eq!(map.get_data("d"), Ok(&[1, 2, 3][..]));
        assert_eq!(map.append_data("d", &[4, 5, 6]), Ok(()));
        assert_eq!(map.get_data("d"), Ok(&[1, 2, 3][..]));
        {
            let iter = map.get_data_iter("d");
            assert!(iter.is_ok());
            let mut iter = iter.unwrap();
            assert_eq!(iter.next(), Some(&[1, 2, 3][..]));
            assert_eq!(iter.next(), Some(&[4, 5, 6][..]));
            assert_eq!(iter.next(), None);
        }

        // TODO: node, frame and function method tests when we can make them.

        assert_eq!(map.delete_key("test_frame"), Ok(()));
        assert_eq!(map.delete_key("test_frame"), Err(map::Error::KeyNotFound));

        assert_eq!(map.error(), None);
        assert_eq!(map.set_error("hello there"), Ok(()));
        assert_eq!(
            map.error().as_ref().map(|x| x.as_ref()),
            Some("hello there")
        );
    }

    #[cfg(feature = "gte-vapoursynth-api-34")]
    #[test]
    fn message_handler() {
        let api = API::get().unwrap();
        let (tx, rx) = channel();

        // Hopefully no one logs anything here and breaks the test.
        api.set_message_handler(move |message_type, message| {
            assert_eq!(tx.send((message_type, message.to_owned())), Ok(()));
        });

        assert_eq!(
            api.log(MessageType::Warning, "test warning message"),
            Ok(())
        );
        assert_eq!(
            rx.recv(),
            Ok((
                MessageType::Warning,
                CString::new("test warning message").unwrap()
            ))
        );

        assert_eq!(api.log(MessageType::Debug, "test debug message"), Ok(()));
        assert_eq!(
            rx.recv(),
            Ok((
                MessageType::Debug,
                CString::new("test debug message").unwrap()
            ))
        );

        assert_eq!(
            api.log(MessageType::Critical, "test critical message"),
            Ok(())
        );
        assert_eq!(
            rx.recv(),
            Ok((
                MessageType::Critical,
                CString::new("test critical message").unwrap()
            ))
        );

        {
            lazy_static! {
                static ref SENDER: Mutex<Option<Sender<(MessageType, CString)>>> = Mutex::new(None);
            }

            let (tx, rx) = channel();
            *SENDER.lock().unwrap() = Some(tx);

            api.set_message_handler_trivial(|message_type, message| {
                let guard = SENDER.lock().unwrap();
                let tx = guard.as_ref().unwrap();
                assert_eq!(tx.send((message_type, message.to_owned())), Ok(()));
            });

            assert_eq!(
                api.log(MessageType::Warning, "test warning message"),
                Ok(())
            );
            assert_eq!(
                rx.recv(),
                Ok((
                    MessageType::Warning,
                    CString::new("test warning message").unwrap()
                ))
            );

            assert_eq!(api.log(MessageType::Debug, "test debug message"), Ok(()));
            assert_eq!(
                rx.recv(),
                Ok((
                    MessageType::Debug,
                    CString::new("test debug message").unwrap()
                ))
            );

            assert_eq!(
                api.log(MessageType::Critical, "test critical message"),
                Ok(())
            );
            assert_eq!(
                rx.recv(),
                Ok((
                    MessageType::Critical,
                    CString::new("test critical message").unwrap()
                ))
            );
        }

        api.clear_message_handler();
    }
}
