use crate::{
	LINODE_CPU_PER_CORE, LINODE_DISK_PER_CORE, NOMAD_RESERVE_MEMORY_MIB,
	PEGBOARD_CONTAINER_RESERVE_MEMORY_MIB, RESERVE_LB_MEMORY_MIB,
};

/// Provider agnostic hardware specs.
#[derive(Debug)]
pub struct ServerSpec {
	pub cpu_cores: u32,
	/// Mhz
	pub cpu: u32,
	/// MiB
	pub memory: u32,
	/// MiB
	pub disk: u32,
	/// Kibps
	pub bandwidth: u32,
}

impl ServerSpec {
	pub fn from_linode(instance_type: &linode::types::InstanceType) -> ServerSpec {
		// Account for kernel memory overhead
		// https://www.linode.com/community/questions/17791/why-doesnt-free-m-match-the-full-amount-of-ram-of-my-nanode-plan
		let memory = instance_type.memory * 95 / 100;
		// Remove reserved resources
		let memory = memory - RESERVE_LB_MEMORY_MIB;

		ServerSpec {
			cpu_cores: instance_type.vcpus,
			cpu: instance_type.vcpus * LINODE_CPU_PER_CORE,
			memory,
			// MB to MiB
			disk: instance_type.disk * 1000 / 1024 * 1000 / 1024,
			// Mbps to Kibps
			bandwidth: instance_type.network_out * 1000 / 1024 * 1000,
		}
	}

	pub fn cpu_per_core(&self) -> u32 {
		LINODE_CPU_PER_CORE
	}

	pub fn memory_per_core_nomad(&self) -> u32 {
		(self.memory - NOMAD_RESERVE_MEMORY_MIB) / self.cpu_cores
	}

	pub fn memory_per_core_pb_container(&self) -> u32 {
		(self.memory - PEGBOARD_CONTAINER_RESERVE_MEMORY_MIB) / self.cpu_cores
	}

	pub fn disk_per_core(&self) -> u32 {
		LINODE_DISK_PER_CORE
	}

	pub fn bandwidth_per_core(&self) -> u32 {
		self.bandwidth / self.cpu_cores
	}
}
