
pub trait ConfigService {
    fn get_locations(&self) -> &[String];
}